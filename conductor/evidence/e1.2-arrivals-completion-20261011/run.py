import subprocess,os,sys,json,hashlib,datetime
from pathlib import Path
label=sys.argv[1];assert label.replace("-", "").isalnum()
argv=sys.argv[2:];out=Path(__file__).resolve().parent
TC="/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin"
env=dict(os.environ,PATH=TC+":"+os.environ["PATH"],RUSTC=TC+"/rustc",RUSTDOC=TC+"/rustdoc",CARGO_HOME="/tmp/c2-route-receipt-cargo-home",CARGO_TARGET_DIR=str(out/"target"),CARGO_INCREMENTAL="0",CARGO_PROFILE_DEV_DEBUG="0",CARGO_PROFILE_TEST_DEBUG="0",CARGO_BUILD_JOBS="2")
rust=subprocess.check_output([TC+"/rustc","--version"],text=True).strip();assert rust.startswith("rustc 1.99.0 ")
def git(*args):return subprocess.check_output(["git",*args],text=True).strip()
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
receipt=dict(argv=argv,cwd=str(Path.cwd()),head=git("rev-parse","HEAD"),dirty=git("status","--porcelain"),rustc=rust,cargo=subprocess.check_output([TC+"/cargo","--version"],text=True).strip(),started=datetime.datetime.now(datetime.timezone.utc).isoformat(),source_sha256=sha("crates/careops-ed/src/arrivals.rs"),lock_sha256=sha("Cargo.lock"))
with (out/(label+".log")).open("w") as f:p=subprocess.run(argv,env=env,stdout=f,stderr=subprocess.STDOUT)
receipt.update(exit=p.returncode,completed=datetime.datetime.now(datetime.timezone.utc).isoformat(),log_sha256=sha(out/(label+".log")))
(out/(label+".json")).write_text(json.dumps(receipt,indent=2)+"\n")
print(json.dumps(receipt));print((out/(label+".log")).read_text()[-2500:]);raise SystemExit(p.returncode)
