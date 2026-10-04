"""Bounded POSIX process IO, accounting and cleanup for development benchmarks."""

from contextlib import contextmanager
import math
import os
import platform
import select
import signal
import subprocess
import time

try:
    import resource
except ImportError:  # pragma: no cover - benchmark host contract is POSIX
    resource = None


MAX_OUTPUT = 8 * 1024 * 1024



MAX_STDERR = 64 * 1024



MAX_DRAIN_CHUNKS_PER_READY = 16



CONSUMER_QUANTUM = 64 * 1024



MAX_WALL_SECONDS = 10.0



MAX_RSS_BYTES = 256 * 1024 * 1024



class BenchmarkError(RuntimeError):
    pass



def _rss_bytes(usage) -> int:
    # Linux reports KiB; Darwin reports bytes.
    return int(usage.ru_maxrss if platform.system() == "Darwin"
               else usage.ru_maxrss * 1024)



def run_bounded(argv: list[str], *, timeout: float = MAX_WALL_SECONDS,
                stdout_limit: int = MAX_OUTPUT,
                slow_consumer_delay: float = 0.0) -> dict:
    """Run one POSIX child, continuously drain pipes, and wait4/reap exactly once."""
    if resource is None or not hasattr(os, "wait4"):
        raise BenchmarkError("POSIX wait4 resource accounting is unavailable")
    if (not isinstance(timeout, (int, float)) or isinstance(timeout, bool)
            or not 0 < timeout <= MAX_WALL_SECONDS or not isinstance(stdout_limit, int)
            or isinstance(stdout_limit, bool) or not 0 <= stdout_limit <= MAX_OUTPUT
            or not isinstance(slow_consumer_delay, (int, float))
            or isinstance(slow_consumer_delay, bool) or not math.isfinite(slow_consumer_delay)
            or slow_consumer_delay < 0 or slow_consumer_delay > timeout):
        raise BenchmarkError("invalid bounded-process limits")
    start = time.monotonic()
    with _managed_child(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            start_new_session=True) as proc:
        assert proc.stdout is not None and proc.stderr is not None
        streams = {}
        for stream, label in ((proc.stdout, "stdout"), (proc.stderr, "stderr")):
            os.set_blocking(stream.fileno(), False)
            streams[stream.fileno()] = (stream, label)
        output = bytearray()
        errors = bytearray()
        overflow = False
        child_status = None
        usage = None
        timed_out = False
        killed = False
        term_at = None
        escalated = False
        eof = set()
        read_count = 0
        paced_bytes = 0
        last_read_at = start
        max_read_gap = 0.0
        try:
            while streams or child_status is None:
                now = time.monotonic()
                elapsed = now - start
                if elapsed >= timeout and not timed_out:
                    timed_out = True
                    _kill_group(proc.pid)
                    killed = True
                    term_at = now
                if killed and term_at is not None and not escalated and now - term_at >= 0.25:
                    # The process may have exited while a descendant still holds a pipe.
                    _kill_group(proc.pid, signal.SIGKILL)
                    escalated = True
                if child_status is None:
                    try:
                        waited, status, child_usage = os.wait4(proc.pid, os.WNOHANG)
                    except ChildProcessError:
                        raise BenchmarkError("child was reaped outside wait4 accounting")
                    if waited:
                        child_status, usage = status, child_usage
                        proc.returncode = os.waitstatus_to_exitcode(status)
                ready, _, _ = select.select(list(streams), [], [], 0.025)
                for fd in ready:
                    stream, label = streams[fd]
                    for _ in range(MAX_DRAIN_CHUNKS_PER_READY):
                        now = time.monotonic()
                        if time.monotonic() - start >= timeout and not timed_out:
                            timed_out = killed = True
                            _kill_group(proc.pid)
                            term_at = time.monotonic()
                        if killed and term_at is not None and not escalated and now - term_at >= 0.25:
                            _kill_group(proc.pid, signal.SIGKILL)
                            escalated = True
                        try:
                            chunk = os.read(fd, 64 * 1024)
                        except BlockingIOError:
                            break
                        if not chunk:
                            streams.pop(fd, None)
                            eof.add(label)
                            break
                        read_at = time.monotonic()
                        read_count += 1
                        max_read_gap = max(max_read_gap, read_at - last_read_at)
                        last_read_at = read_at
                        if label == "stdout":
                            remaining = max(0, stdout_limit - len(output))
                            output.extend(chunk[:remaining])
                            overflow |= len(chunk) > remaining
                            paced_bytes += len(chunk)
                            if slow_consumer_delay and paced_bytes >= CONSUMER_QUANTUM:
                                paced_bytes %= CONSUMER_QUANTUM
                                remaining_time = timeout - (time.monotonic() - start)
                                if remaining_time > 0:
                                    time.sleep(min(slow_consumer_delay, remaining_time))
                            if overflow and not killed:
                                _kill_group(proc.pid)
                                killed = True
                                term_at = time.monotonic()
                        else:
                            remaining = max(0, MAX_STDERR - len(errors))
                            errors.extend(chunk[:remaining])
            if usage is None:
                # A wait4 race can only occur if an external owner reaped this child.
                raise BenchmarkError("child status was unavailable to wait4")
            if not isinstance(rss := _rss_bytes(usage), int) or rss <= 0:
                raise BenchmarkError("child peak-RSS reading was invalid")
        except BaseException:
            _kill_group(proc.pid, signal.SIGKILL)
            if child_status is None:
                try:
                    _, status, usage = os.wait4(proc.pid, 0)
                    child_status, proc.returncode = status, os.waitstatus_to_exitcode(status)
                except ChildProcessError:
                    pass
            raise
        finally:
            proc.stdout.close()
            proc.stderr.close()
        elapsed = time.monotonic() - start
        exit_code = os.waitstatus_to_exitcode(child_status)
        rss = _rss_bytes(usage)
        return {"argv": argv, "exit_code": exit_code, "elapsed_seconds": elapsed,
                "peak_rss_bytes": rss, "stdout": bytes(output), "stderr": bytes(errors),
                "stdout_overflow": overflow, "timed_out": timed_out, "killed": killed,
                "stdout_read_count": read_count,
                "max_read_gap_seconds": max_read_gap}



@contextmanager
def _managed_child(argv, **kwargs):
    """Own the process group from spawn through setup, execution and cleanup."""
    kwargs["start_new_session"] = True
    proc = subprocess.Popen(argv, **kwargs)
    try:
        yield proc
    finally:
        # A reaped parent can leave descendants holding pipes; terminate the
        # owned group even when Popen.returncode has already been populated.
        _kill_group(proc.pid, signal.SIGKILL)
        if proc.returncode is None:
            try:
                _, status, _ = os.wait4(proc.pid, 0)
                proc.returncode = os.waitstatus_to_exitcode(status)
            except ChildProcessError:
                pass  # Already reaped; never fabricate a resource measurement.
        for stream in (proc.stdin, proc.stdout, proc.stderr):
            if stream is not None:
                stream.close()



def _kill_group(pid: int, sig=signal.SIGTERM) -> None:
    try:
        os.killpg(pid, sig)
    except ProcessLookupError:
        pass



def _run_to_full(argv: list[str]) -> dict:
    """Run a child with stdout genuinely redirected to /dev/full; bound stderr/wait4."""
    if resource is None or not hasattr(os, "wait4"):
        raise BenchmarkError("POSIX wait4 resource accounting is unavailable")
    sink_path = Path("/dev/full")
    start = time.monotonic()
    errors = bytearray()
    overflow = False
    status = usage = None
    timed_out = False
    escalated = False
    term_at = None
    with sink_path.open("wb") as sink:
        with _managed_child(argv, stdout=sink, stderr=subprocess.PIPE,
                                start_new_session=True) as proc:
            assert proc.stderr is not None
            fd = proc.stderr.fileno()
            os.set_blocking(fd, False)
            eof = False
            try:
                while status is None or not eof:
                    now = time.monotonic()
                    if now - start >= MAX_WALL_SECONDS and not timed_out:
                        timed_out = True
                        _kill_group(proc.pid)
                        term_at = now
                    if term_at is not None and not escalated and now - term_at >= 0.25:
                        _kill_group(proc.pid, signal.SIGKILL)
                        escalated = True
                    if status is None:
                        waited, child_status, child_usage = os.wait4(proc.pid, os.WNOHANG)
                        if waited:
                            status, usage = child_status, child_usage
                            proc.returncode = os.waitstatus_to_exitcode(status)
                    ready, _, _ = select.select([fd] if not eof else [], [], [], 0.025)
                    if ready:
                        for _ in range(MAX_DRAIN_CHUNKS_PER_READY):
                            try:
                                chunk = os.read(fd, 64 * 1024)
                            except BlockingIOError:
                                break
                            if not chunk:
                                eof = True
                                break
                            remaining = max(0, MAX_STDERR - len(errors))
                            errors.extend(chunk[:remaining])
                            overflow |= len(chunk) > remaining
                if usage is None:
                    raise BenchmarkError("/dev/full child wait4 accounting missing")
            except BaseException:
                _kill_group(proc.pid, signal.SIGKILL)
                if status is None:
                    try:
                        _, status, usage = os.wait4(proc.pid, 0)
                        proc.returncode = os.waitstatus_to_exitcode(status)
                    except ChildProcessError:
                        pass
                raise
            finally:
                proc.stderr.close()
        return {"exit_code": os.waitstatus_to_exitcode(status),
                "elapsed_seconds": time.monotonic() - start,
                "peak_rss_bytes": _rss_bytes(usage), "stderr": bytes(errors),
                "stderr_overflow": overflow, "timed_out": timed_out}
