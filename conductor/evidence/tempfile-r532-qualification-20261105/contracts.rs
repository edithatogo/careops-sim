// Apache-2.0. Filesystem outcomes independent of naming implementation.
mod experiment;
mod checkpoint_seam;
use std::{env,fs,path::Path,io::{self,Write},collections::HashSet,thread};
use experiment::{baseline,candidate,error::{ExperimentError,ExperimentErrorKind}};
fn count(p:&Path)->usize {fs::read_dir(p).unwrap().count()}
fn main() {
    let a:Vec<_>=env::args().collect();let root=Path::new(&a[1]);fs::create_dir_all(root).unwrap();let mut cases=0;
    let writers:[(&str,fn(&Path,&[u8])->Result<(),ExperimentError>);2]=[("baseline",baseline::atomic_write_file),("candidate",candidate::atomic_write_file)];
    for (name,writer) in writers {
        let r=root.join(name);fs::create_dir(&r).unwrap();let dest=r.join("nested/result.bin");
        writer(&dest,b"first").unwrap();assert_eq!(fs::read(&dest).unwrap(),b"first");cases+=1;
        writer(&dest,b"replacement").unwrap();assert_eq!(fs::read(&dest).unwrap(),b"replacement");cases+=1;
        let bad=r.join("directory");fs::create_dir(&bad).unwrap();assert_eq!(writer(&bad,b"new").unwrap_err().kind,ExperimentErrorKind::OutputWriteFailed);
        assert!(bad.is_dir());assert_eq!(count(&r),2);cases+=1;
        let parent=r.join("parent-file");fs::write(&parent,b"old").unwrap();
        assert_eq!(writer(&parent.join("bad"),b"x").unwrap_err().kind,ExperimentErrorKind::OutputWriteFailed);assert_eq!(fs::read(&parent).unwrap(),b"old");cases+=1;
        let bare=if name=="baseline" {"bare-baseline"} else {"bare-candidate"};writer(Path::new(bare),b"base").unwrap();assert_eq!(fs::read(bare).unwrap(),b"base");cases+=1;
        let dest=r.join("sentinel-target");let stage=dest.with_extension(format!("tmp.{}",std::process::id()));fs::write(&stage,b"SENTINEL").unwrap();
        writer(&dest,b"replacement").unwrap();assert_eq!(fs::read(&dest).unwrap(),b"replacement");
        if name=="candidate" {assert_eq!(fs::read(stage).unwrap(),b"SENTINEL");} else {assert!(!stage.exists());}cases+=1;
    }
    let mut handles=vec![];
    for i in 0..8 {let root=root.to_path_buf();handles.push(thread::spawn(move|| {
        let mut builder=tempfile::Builder::new();
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;builder.permissions(fs::Permissions::from_mode(0o700));}
        let dir=builder.prefix("r532-").tempdir_in(root).unwrap();let p=dir.path().to_path_buf();
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;assert_eq!(fs::metadata(&p).unwrap().permissions().mode()&0o777,0o700);}
        fs::write(p.join("data"),[i]).unwrap();assert_eq!(fs::read(p.join("data")).unwrap(),[i]);(dir,p)
    }));}
    let owners:Vec<_>=handles.into_iter().map(|h|h.join().unwrap()).collect();assert_eq!(owners.iter().map(|(_,p)|p.clone()).collect::<HashSet<_>>().len(),8);
    let paths:Vec<_>=owners.iter().map(|(_,p)|p.clone()).collect();drop(owners);assert!(paths.iter().all(|p|!p.exists()));cases+=1;
    let before=count(root);let result:Result<(),io::Error>=(|| {let d=tempfile::tempdir_in(root)?;fs::write(d.path().join("data"),b"x")?;Err(io::Error::other("intentional"))})();assert!(result.is_err());assert_eq!(count(root),before);cases+=1;
    let keep=tempfile::tempdir_in(root).unwrap().keep();assert!(keep.is_dir());fs::remove_dir(keep).unwrap();cases+=1;
    let dest=root.join("protected");fs::write(&dest,b"original").unwrap();let mut stage=tempfile::NamedTempFile::new_in(root).unwrap();stage.write_all(b"new").unwrap();let scratch=stage.path().to_path_buf();
    let failure=stage.persist_noclobber(&dest).unwrap_err();assert_eq!(fs::read(dest).unwrap(),b"original");drop(failure);assert!(!scratch.exists());cases+=1;
    let dest=root.join("postpersist");fs::write(&dest,b"old").unwrap();
    let syncs=std::sync::atomic::AtomicUsize::new(0);
    let result=checkpoint_seam::durable_checkpoint_with_sync(&dest,b"published",&|file| {
        if syncs.fetch_add(1,std::sync::atomic::Ordering::SeqCst)==1 {Err(io::Error::other("injected directory sync failure"))} else {file.sync_all()}
    });
    assert!(result.is_err());assert_eq!(syncs.load(std::sync::atomic::Ordering::SeqCst),2);assert_eq!(fs::read(dest).unwrap(),b"published");cases+=1;
    println!("{{\"cases\":{},\"baseline_pid_sentinel_destroyed\":true,\"candidate_pid_sentinel_preserved\":true,\"ordinary_raii_only\":true,\"postpersist_error_not_rollback\":true}}",cases);
}
