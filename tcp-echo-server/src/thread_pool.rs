use std::{
    sync::{Arc, Mutex, mpsc::Sender},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_sender: std::sync::mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> ThreadPool {
        let (job_sender, job_receiver) = std::sync::mpsc::channel::<Job>();

        let shareable_receiver = Arc::new(Mutex::new(job_receiver));

        let workers = (0..num_threads)
            .map(|id| Worker::new(id, shareable_receiver.clone()))
            .collect();

        ThreadPool {
            workers,
            job_sender,
        }
    }

    pub fn execute(&self, job: Job) {
        if let Err(x) = self.job_sender.send(job) {
            println!("Error while sending job to worker: {}", x);
        }
    }
}

pub struct Worker {
    id: usize,
    thread: std::thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, job_receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            thread: thread::spawn(move || {
                println!("Worker {} started", id);

                loop {
                    {
                        if let Ok(locked_receiver) = job_receiver.try_lock()
                            && let Ok(job) = locked_receiver.recv()
                        {
                            println!("Worker {} got a job", id);
                            job();
                        }
                    }
                }
            }),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
