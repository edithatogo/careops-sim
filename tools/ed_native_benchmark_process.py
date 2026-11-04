"""Native CLI benchmark process contract v1.

Reuse the qualified, immutable POSIX child manager; its wall/RSS/output budgets,
TERM/KILL process-group cleanup and exactly-once wait4 accounting are unchanged.
"""
from ed_benchmark_process import (
    BenchmarkError, MAX_OUTPUT, MAX_STDERR, MAX_RSS_BYTES, MAX_WALL_SECONDS,
    CONSUMER_QUANTUM, run_bounded, _managed_child, _kill_group, _rss_bytes,
)
PROTOCOL = 'careops.ed.native-benchmark-process.v1'


def unread_stdout_termination(argv):
    """Real full unread pipe, live child, TERM then bounded KILL, one wait4 reap."""
    import os
    import select
    import signal
    import subprocess
    import time
    started=time.monotonic()
    overall_deadline=started+MAX_WALL_SECONDS
    with _managed_child(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE) as proc:
        # Artifact publication precedes CSV. Reserve the observation and TERM
        # grace inside the original total wall budget, not a separate startup cap.
        ready,_,_=select.select([proc.stdout],[],[],max(0.0,overall_deadline-time.monotonic()-0.7))
        if not ready or proc.poll() is not None:
            raise BenchmarkError('native CSV child did not produce live unread output')
        time.sleep(0.1)
        if proc.poll() is not None:
            raise BenchmarkError('native CSV child completed while stdout remained unread')
        _kill_group(proc.pid)
        deadline=min(time.monotonic()+0.5,overall_deadline-0.1); escalated=False
        while True:
            waited,status,usage=os.wait4(proc.pid,os.WNOHANG)
            if waited: break
            if time.monotonic()>=deadline:
                _kill_group(proc.pid,signal.SIGKILL);escalated=True
                _,status,usage=os.wait4(proc.pid,0);break
            time.sleep(0.01)
        proc.returncode=os.waitstatus_to_exitcode(status)
        elapsed=time.monotonic()-started;rss=_rss_bytes(usage)
        if proc.returncode==0 or elapsed>MAX_WALL_SECONDS or rss<=0 or rss>MAX_RSS_BYTES:
            raise BenchmarkError('unread native CSV termination exceeded exit/wall/RSS gate')
        return {'status':'pass','stdout_read':False,'observed_live_unread_pipe':True,
                'exit_code':proc.returncode,'peak_rss_bytes':rss,'elapsed_seconds':elapsed,
                'term_sent':True,'kill_escalated':escalated,'wait4_reaps':1}
