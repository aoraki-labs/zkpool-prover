use std::thread::{self, JoinHandle};
use std::sync::{Arc, mpsc, Mutex};

type Job = Box<dyn FnOnce() + 'static + Send>;
enum Message {
    ByeBye,
    NewJob(Job),
}


struct Worker where
{
    id: usize,
    t: Option<JoinHandle<()>>,
}

impl Worker
{
    fn new(id: usize, receiver: Arc::<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let t = thread::spawn( move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("do job from worker[{}]", id);
                        job();
                    },
                    Message::ByeBye => {
                        println!("ByeBye from worker[{}]", id);
                        break
                    },
                }  
            }
        });

        Worker {
            id,
            t: Some(t),
        }
    }
}

pub struct Pool {
    workers: Vec<Worker>,
    max_workers: usize,
    sender: Arc<Mutex<mpsc::Sender<Message>>>
}

impl Pool where {
    pub fn new(max_workers: usize) -> Pool {
        if max_workers == 0 {
            panic!("max_workers must be greater than zero!")
        }
        let (tx, rx) = mpsc::channel();

        let mut workers = Vec::with_capacity(max_workers);
        let receiver = Arc::new(Mutex::new(rx));
        for i in 0..max_workers {
            workers.push(Worker::new(i, Arc::clone(&receiver))); //start the worker
        }

        Pool { workers, max_workers, sender: Arc::new(Mutex::new(tx)) }
    }

    pub fn execute<F>(&self, f:F) where F: FnOnce() + 'static + Send  //send the task
    {

        let job = Message::NewJob(Box::new(f));
        let sender = self.sender.clone();
        let sender = sender.lock().unwrap();
        sender.send(job).unwrap()
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        for _ in 0..self.max_workers {
            let sender = self.sender.clone();
            let sender = sender.lock().unwrap();
            sender.send(Message::ByeBye).unwrap();
            // self.sender.send(Message::ByeBye).unwrap();
        }
        for w in self.workers.iter_mut() {
            if let Some(t) = w.t.take() {
                t.join().unwrap();
            }
        }
    }

    #[cfg(test)]
    mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let p = Pool::new(4);
        p.execute(|| println!("do new job1"));
        p.execute(|| println!("do new job2"));
        p.execute(|| println!("do new job3"));
        p.execute(|| println!("do new job4"));
     }
    }

}



