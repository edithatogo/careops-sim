// Apache-2.0. Same page-cache IO harness for baseline and candidate.
mod experiment;
use std::{env, path::PathBuf, time::Instant};
#[cfg(feature="candidate")]
use experiment::candidate::atomic_write_file;
#[cfg(not(feature="candidate"))]
use experiment::baseline::atomic_write_file;
fn main() {
    let a:Vec<_>=env::args().collect();let path=PathBuf::from(&a[2]);
    let size:usize=a[3].parse().unwrap();let bytes:Vec<u8>=(0..size).map(|i|(i%251) as u8).collect();
    if a[1]=="once" {
        match atomic_write_file(&path,&bytes) {Ok(())=>println!("ok"),Err(e)=>println!("err:{}",e.kind)}
        return;
    }
    let count:usize=a[4].parse().unwrap();for _ in 0..8 {atomic_write_file(&path,&bytes).unwrap();}
    let start=Instant::now();for _ in 0..count {atomic_write_file(&path,&bytes).unwrap();}
    println!("{{\"ops\":{},\"elapsed_ns\":{}}}",count,start.elapsed().as_nanos());
}
