use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub enum PoolMessage<T> {
    Task(Box<dyn FnOnce() -> T + Send + 'static>),
    Shutdown,
}

pub struct ThreadPool<T> {
    workers: Vec<thread::JoinHandle<()>>,
    sender: mpsc::Sender<PoolMessage<T>>,
    result_sender: Arc<Mutex<mpsc::Sender<T>>>,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new(size: usize) -> (ThreadPool<T>, mpsc::Receiver<T>) {
        println!("Creating new thread pool with {} threads", size);
        let (task_sender, task_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let task_receiver = Arc::new(Mutex::new(task_receiver));
        let result_sender = Arc::new(Mutex::new(result_sender));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            println!("Creating worker {}", id);
            let task_receiver = Arc::clone(&task_receiver);
            let result_sender = Arc::clone(&result_sender);
            
            let handle = thread::spawn(move || {
                println!("Worker {} started", id);
                loop {
                    println!("Worker {} waiting for task", id);
                    let message = task_receiver.lock().unwrap().recv().unwrap();
                    match message {
                        PoolMessage::Task(callback) => {
                            println!("Worker {} processing task", id);
                            let result = callback();
                            println!("Worker {} has result, about to send", id);
                            let send_result = result_sender.lock().unwrap().send(result);
                            println!("Send attempt completed");
                            match send_result {
                                Ok(_) => println!("Worker {} successfully sent result", id),
                                Err(e) => println!("Worker {} send failed: {:?}", id, e),
                            }
                        },
                        PoolMessage::Shutdown => {
                            println!("Worker {} shutting down", id);
                            break;
                        }
                    }
                }
            });
            workers.push(handle);
        }

        (
            ThreadPool {
                workers,
                sender: task_sender,
                result_sender: Arc::clone(&result_sender),
            },
            result_receiver,
        )
    }

    pub fn execute(&self, callback: Box<dyn FnOnce() -> T + Send + 'static>) {
        self.sender.send(PoolMessage::Task(callback)).unwrap();
    }

    fn shutdown(self) {
        for _ in 0..self.workers.len() {
            self.sender.send(PoolMessage::Shutdown).unwrap();
        }
        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}