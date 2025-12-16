use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;

use crate::progress::{Progress, FileStatus};
use crate::types::{FileAnalysis, FileStats};
use crate::errors::ProcessingError;

#[derive(Clone)]
pub struct Processor {
    progress: Arc<Mutex<Progress>>,
    cancel: Arc<AtomicBool>,
}

impl Processor {
    pub fn new(progress: Arc<Mutex<Progress>>, cancel: Arc<AtomicBool>) -> Self {
        Processor { progress, cancel }
    }

    pub fn process_file(&self, path: PathBuf) -> FileAnalysis {
        let filename = path.to_string_lossy().to_string();

        {
            let mut pr = self.progress.lock().unwrap();
            pr.statuses.insert(filename.clone(), FileStatus::Processing);
        }

        let start = Instant::now();
        let mut analysis = FileAnalysis {
            filename: filename.clone(),
            stats: FileStats {
                word_count: 0,
                line_count: 0,
                char_frequencies: HashMap::new(),
                size_bytes: 0,
            },
            errors: Vec::new(),
            processing_time: start.elapsed(),
        };

        if self.cancel.load(Ordering::SeqCst) {
            let mut pr = self.progress.lock().unwrap();
            pr.statuses.insert(filename.clone(), FileStatus::Cancelled);
            return analysis;
        }

        match fs::read_to_string(&path) {
            Ok(contents) => {
                analysis.stats.size_bytes = contents.len() as u64;
                analysis.stats.line_count = contents.lines().count();
                analysis.stats.word_count = contents.split_whitespace().count();

                for c in contents.chars() {
                    *analysis.stats.char_frequencies.entry(c).or_insert(0) += 1;
                }
            }
            Err(e) => {
                analysis.errors.push(ProcessingError {
                    message: format!("Failed to read file {}: {}", filename, e),
                });
            }
        }

        analysis.processing_time = start.elapsed();

        // Update progress
        {
            let mut pr = self.progress.lock().unwrap();
            pr.processed += 1;
            if analysis.errors.is_empty() {
                pr.statuses.insert(filename.clone(), FileStatus::Done);
            } else {
                pr.statuses.insert(filename.clone(), FileStatus::Error);
            }
        }

        analysis
    }
}

// Save results to a text file
pub fn save_results_txt<P: AsRef<Path>>(path: P, analyses: &[FileAnalysis]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for analysis in analyses {
        writeln!(writer, "File: {}", analysis.filename)?;
        writeln!(writer, "  Size (bytes): {}", analysis.stats.size_bytes)?;
        writeln!(writer, "  Words: {}", analysis.stats.word_count)?;
        writeln!(writer, "  Lines: {}", analysis.stats.line_count)?;
        writeln!(writer, "  Character frequencies:")?;
        for (ch, count) in &analysis.stats.char_frequencies {
            writeln!(writer, "    '{}': {}", ch, count)?;
        }
        if !analysis.errors.is_empty() {
            writeln!(writer, "  Errors:")?;
            for e in &analysis.errors {
                writeln!(writer, "    {}", e)?;
            }
        }
        writeln!(writer, "  Processing time: {:.2?}\n", analysis.processing_time)?;
    }

    Ok(())
}
