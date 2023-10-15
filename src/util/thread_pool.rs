use std::{
    io::{Error, Result as IoResult},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use PoolCreationError as PCE;

type ReceiverArc = Arc<Mutex<mpsc::Receiver<Job>>>;
type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

#[derive(Debug)]
pub enum PoolCreationError {
    ZeroThreads,
    FailedSpawn(Error),
}

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: ReceiverArc) -> IoResult<Self> {
        let builder = thread::Builder::new();
        let thread = builder.spawn(move || {
            loop {
                let msg = receiver.lock().unwrap().recv();

                if msg.is_err() {
                    break;
                }

                let job = msg.unwrap();
                job();
            }
        })?;

        Ok(Self { id, thread: Some(thread) })
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

impl ThreadPool {
    pub fn build(num_threads: usize) -> Result<Self, PoolCreationError> {
        if num_threads == 0 {
            return Err(PCE::ZeroThreads);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            match Worker::new(id, Arc::clone(&receiver)) {
                Ok(worker) => workers.push(worker),
                Err(e) => return Err(PCE::FailedSpawn(e)),
            }
        }

        Ok(Self { workers, sender: Some(sender) })
    }

    pub fn execute<F>(&self, f: F)
        where
        F: FnOnce() + Send + 'static,
        {
            self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
        }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            worker.join();
        }
    }
}
