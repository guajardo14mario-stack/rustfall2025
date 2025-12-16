use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::time::Instant;

use parallel_file_processor::progress::{Progress, FileStatus};
use parallel_file_processor::processor::{Processor, save_results_txt};
use parallel_file_processor::thread_pool::ThreadPool;
use parallel_file_processor::types::FileAnalysis;

fn collect_files(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <data_dir> <threads>", args[0]);
        return;
    }

    let data_dir = PathBuf::from(&args[1]);
    let threads: usize = args[2].parse().unwrap_or(4);

    let mut files = Vec::new();
    collect_files(&data_dir, &mut files);
    println!("Found {} files to process.", files.len());

    let progress = Arc::new(Mutex::new(Progress::new(files.len())));
    {
        let mut pr = progress.lock().unwrap();
        for f in &files {
            pr.statuses.insert(f.to_string_lossy().to_string(), FileStatus::Pending);
        }
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let pool = ThreadPool::new(threads);
    let processor = Processor::new(progress.clone(), cancel_flag.clone());

    let results: Arc<Mutex<Vec<FileAnalysis>>> = Arc::new(Mutex::new(Vec::new()));
    let start_time = Instant::now();

    for file in files {
        let processor_clone = processor.clone();
        let results_clone = results.clone();
        pool.execute(move || {
            let analysis = processor_clone.process_file(file);
            let mut res = results_clone.lock().unwrap();
            res.push(analysis);
        });
    }

    loop {
        let pr = progress.lock().unwrap();
        if pr.processed >= pr.total_files {
            break;
        }
        drop(pr);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let elapsed = start_time.elapsed();
    println!("All files processed in {:.2?}", elapsed);

    let pr = progress.lock().unwrap();
    for (filename, status) in &pr.statuses {
        println!("{}: {:?}", filename, status);
    }

    let res = results.lock().unwrap();
    if let Err(e) = save_results_txt("data/results.txt", &res) {
        eprintln!("Failed to save results: {}", e);
    } else {
        println!("All results saved to data/results.txt");
    }
}
