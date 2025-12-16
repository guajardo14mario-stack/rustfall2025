use std::time::Instant;
use parallel_file_processor::thread_pool::ThreadPool;
use parallel_file_processor::processor::Processor;
use parallel_file_processor::progress::Progress;
use std::sync::{Arc, atomic::AtomicBool};

fn main() {
    // Point this to a folder with your many book files (plain text). Run with `cargo run --release --bin simple_bench <dir> <threads>`
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: bench <dir> <threads>");
        std::process::exit(1);
    }
    let dir = std::path::PathBuf::from(&args[1]);
    let threads: usize = args[2].parse().unwrap_or(4);

    let mut files = Vec::new();
    fn visit(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() { visit(&p, out); } else { out.push(p); }
            }
        }
    }
    visit(&dir, &mut files);

    let progress = Arc::new(std::sync::Mutex::new(Progress::new(files.len())));
    {
        let mut p = progress.lock().unwrap();
        for f in &files {
            p.statuses.insert(f.to_string_lossy().to_string(), parallel_file_processor::progress::FileStatus::Pending);
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let pool = ThreadPool::new(threads);
    let proc = Processor::new(progress.clone(), cancel.clone());

    let start = Instant::now();
    let _ = proc.process_files(files, &pool);
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
}
