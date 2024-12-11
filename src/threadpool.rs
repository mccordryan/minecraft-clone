use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use queues::Queue;

pub enum PoolMessage<T> {
   // callback function:
    Task(Box<dyn FnOnce() -> T + Send + 'static>), 
    Shutdown,      // Signal to shutdown worker
}

pub struct ThreadPool<T> {
    workers: Vec<thread::JoinHandle<()>>,
    sender: mpsc::Sender<PoolMessage<T>>,
}
impl <T: Send + 'static> ThreadPool<T> {
    pub fn new(size: usize) -> (ThreadPool<T>, mpsc::Receiver<T>) {
        println!("Creating new thread pool with {} threads", size);
        let (task_sender, task_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let task_receiver = Arc::new(Mutex::new(task_receiver));
        let result_sender = Arc::new(result_sender);
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            println!("Creating worker {}", id);
            let task_receiver = Arc::clone(&task_receiver);
            let result_sender = Arc::clone(&result_sender);
            let handle = thread::spawn(move || loop {
                let message = task_receiver.lock().unwrap().recv().unwrap();
                match message {
                    PoolMessage::Task(callback) => {
                        println!("Worker {} starting task execution", id);
                        let result = callback();
                        println!("Worker {} finished task execution, sending result", id);
                        result_sender.send(result).unwrap();
                        println!("Worker {} successfully sent result", id);
                    },
                    PoolMessage::Shutdown => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            });
            workers.push(handle);
        }

        (ThreadPool { workers, sender: task_sender }, result_receiver)
    }

    pub fn execute(&self, callback: Box<dyn FnOnce() -> T + Send + 'static>) {
        // println!("Executing new task");
        self.sender.send(PoolMessage::Task(callback)).unwrap();
    }



    fn shutdown(self) {
        // Send shutdown message to all workers
        for _ in 0..self.workers.len() {
            self.sender.send(PoolMessage::Shutdown).unwrap();
        }

        // Wait for all workers to complete
        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

