use std::collections::HashMap;

#[derive(Debug)]
pub enum FileStatus {
    Pending,
    Processing,
    Done,
    Error,
    Cancelled,
}

pub struct Progress {
    pub total_files: usize,
    pub processed: usize,
    pub statuses: HashMap<String, FileStatus>,
}

impl Progress {
    pub fn new(total_files: usize) -> Self {
        Progress {
            total_files,
            processed: 0,
            statuses: HashMap::new(),
        }
    }
}
