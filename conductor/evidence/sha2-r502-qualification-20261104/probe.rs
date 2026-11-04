// Apache-2.0. Isolated R50.2 probe; never linked into CareOps runtime.
#[cfg(not(feature = "candidate"))]
include!(env!("R502_BASELINE"));
#[cfg(feature = "candidate")]
fn compute_sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args[1].as_str() {
        "digest" => println!("{}", compute_sha256_hex(&std::fs::read(&args[2]).unwrap())),
        "bench" => {
            let n: usize = args[2].parse().unwrap();
            let count: usize = args[3].parse().unwrap();
            let data: Vec<_> = (0..n).map(|i| ((i * 131 + 17) % 256) as u8).collect();
            for _ in 0..128 { std::hint::black_box(compute_sha256_hex(std::hint::black_box(&data))); }
            let start = std::time::Instant::now();
            for _ in 0..count { std::hint::black_box(compute_sha256_hex(std::hint::black_box(&data))); }
            println!("{}", start.elapsed().as_nanos());
        }
        _ => panic!("unknown probe operation"),
    }
}
