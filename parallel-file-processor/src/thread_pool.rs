use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    queue: Arc<(Mutex<VecDeque<Job>>, Condvar)>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let queue = Arc::new((
            Mutex::new(VecDeque::<Job>::new()), 
            Condvar::new()
        ));
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::new();
        for _ in 0..num_threads {
            let q = queue.clone();
            let s = shutdown.clone();
            workers.push(thread::spawn(move || loop {
                let job = {
                    let (lock, cvar) = &*q;
                    let mut jobs = lock.lock().unwrap();
                    while jobs.is_empty() && !s.load(Ordering::SeqCst) {
                        jobs = cvar.wait(jobs).unwrap();
                    }
                    jobs.pop_front()
                };
                match job {
                    Some(task) => task(),
                    None if s.load(Ordering::SeqCst) => break,
                    None => {}
                }
            }));
        }
        Self {workers, queue, shutdown}
    }
    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (lock, cvar) = &*self.queue;
        let mut q = lock.lock().unwrap();
        q.push_back(Box::new(job));
        cvar.notify_one();
    }
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let (_, cvar) = &*self.queue;
        cvar.notify_all();
        while let Some(w) = self.workers.pop() {
            let _ = w.join();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}