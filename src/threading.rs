use std::sync::{Condvar, Arc, Mutex, MutexGuard, LockResult};
use std::thread::{self, JoinHandle}; 
use std::collections::HashMap;
use crossbeam::{select, channel};
use channel::{unbounded, Receiver, Sender};

pub type JobType = Box<dyn FnOnce() + Send + 'static>;
pub type CacheStateType = Arc<Mutex<HashMap<CacheKey, Arc<(Mutex<CacheState>, Condvar)>>>>;
pub type CacheType = Arc<Mutex<HashMap<CacheKey, u8>>>;

enum WorkMsg {
    Work(u8),
    Exit,
}

#[derive(Debug, Eq, PartialEq)]
enum WorkPerformed {
    FromCache,
    New,
}

#[derive(Debug, Eq, PartialEq)]
enum CacheState {
    Ready,
    WorkInProgress,
}

enum ResultMsg {
    Result(u8, WorkPerformed),
    Exited,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<JobType>,
}

impl ThreadPool {
    
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = unbounded();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            self.sender.send(job).unwrap();
        } 
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<JobType>>>) -> Worker {
        let thread: JoinHandle<()> = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {id} got a job; executing.");
            job();
        });
        Worker { id, thread }
    }
}

#[derive(Eq, Hash, PartialEq)]
struct CacheKey(u8);


pub struct CacheJob {
    pool: ThreadPool,
    work_sender: Sender<WorkMsg>,
    work_receiver: Receiver<WorkMsg>,
    result_sender: Sender<ResultMsg>,
    result_receiver: Receiver<ResultMsg>,
    pool_result_sender: Sender<()>,
    pool_result_receiver: Receiver<()>,
    ongoing_work: i32,
    exiting: bool,
    cache:  CacheType,
    cache_state: CacheStateType,
}

impl CacheJob {
    pub fn new() -> CacheJob {
        let pool = ThreadPool::new(2);
        let (work_sender, work_receiver) =  unbounded();
        let (result_sender, result_receiver) =  unbounded();
        let (pool_result_sender, pool_result_receiver) =  unbounded();
        let mut ongoing_work = 0;
        let mut exiting = false;
        let cache: CacheType = Arc::new(Mutex::new(HashMap::new()));
        let cache_state: CacheStateType = Arc::new(Mutex::new(HashMap::new()));
        CacheJob {pool, work_sender, work_receiver, result_sender, result_receiver, pool_result_sender, pool_result_receiver, ongoing_work, exiting, cache, cache_state}
    }

 
    pub fn start(self) {
        let work_receiver = self.work_receiver;

        let _ = thread::spawn(move || loop  {
            select! {
                recv(self.work_receiver) -> msg => {                    
                    match msg {
                        Ok(WorkMsg::Work(key)) => {

                            self.ongoing_work += 1;

                            self.pool.execute(move || {
                                let key = {
                                    let (lock, cvar) = self.get_relevant_cache(key);
                                    let state = self.wait_until_state_is_ready(lock, cvar);
                                    let (key, result) = self.get_result_from_cache(key);
                                    
                                    if let Some(result) = result {
                                        let _ = self.result_sender.send(ResultMsg::Result(result, WorkPerformed::FromCache));
                                        let _ = self.pool_result_sender.send(());
                                        cvar.notify_one();
                                        return;
                                    } else {
                                        state = CacheState::WorkInProgress;
                                        key
                                    }
                                };

                                let _ = self.result_sender.send(ResultMsg::Result(key.clone(), WorkPerformed::New));

                                self.insert_result_into_cache(key);

                                let (lock, cvar) = self.get_cache_state(key);

                                let mut state = lock.lock().unwrap();

                                // Here, since we've set it earlier,
                                // and any other worker would wait
                                // on the state to switch back to ready,
                                // we can be certain the state is "in-progress".
                                assert_eq!(*state, CacheState::WorkInProgress);

                                // Switch the state to ready.
                                *state = CacheState::Ready;

                                // Notify the waiting thread, if any, that the state has changed.
                                // This can be done while still inside the critical section.
                                cvar.notify_one();

                                let _ = self.pool_result_sender.send(());

                            });  
                        },
                        
                        Ok(WorkMsg::Exit) => {
                            self.exiting = true;

                            if self.ongoing_work == 0 {
                                let _ = self.result_sender.send(ResultMsg::Exited);
                                break;
                            }
                        },
                        _ => panic!("Error Receiving a WorkMsg"),
                    }
                },

                recv(self.pool_result_receiver) -> _ => {
                    if self.ongoing_work == 0 {
                        panic!("Received an unexpected pool result.");                
                    }
                    self.ongoing_work -1;
                    if self.ongoing_work == 0 && self.exiting {
                        let _ = self.result_sender.send(ResultMsg::Exited);
                        break;
                    }
                }
            }    
        });

        // assert work performed
        let mut counter = 0;

        // A new counter for work on 1.
        let mut work_one_counter = 0;
    
        loop {
            match self.result_receiver.recv() {
                Ok(ResultMsg::Result(key, cached)) => {
                    counter += 1;
    
                    if key == 1 {
                        work_one_counter += 1;
                    }
    
                    // Now we can assert that by the time
                    // the second result for 1 has been received,
                    // it came from the cache.
                    if key == 1 && work_one_counter == 2 {
                        assert_eq!(cached, WorkPerformed::FromCache);
                    }
                }
                Ok(ResultMsg::Exited) => {
                    assert_eq!(3, counter);
                    break;
                }
                _ => panic!("Error receiving a ResultMsg."),
            }
        }

    }

    fn get_relevant_cache(self, key:u8) -> (&'static Mutex<CacheKey>, &'static Condvar) {
            let (lock, cvar) = {
            let mut state_map =  self.cache_state.lock().unwrap();
            &*state_map
                .entry(CacheKey(key.clone()))
                .or_insert_with(|| {
                    Arc::new((
                        Mutex::new(CacheState::Ready),
                        Condvar::new(),
                    ))
                })
                .clone()
            };
            (lock, cvar)
    }

    fn wait_until_state_is_ready(self, lock: &'static Mutex<CacheKey>, cvar: &'static Condvar) -> Mutex<CacheKey> {
        let mut state = lock.lock().unwrap();

        while let CacheState::WorkInProgress = *state {
            let current_state = cvar
                .wait(state)
                .unwrap();
            state = current_state;
        }

        assert_eq!(*state, CacheState::Ready);
        state
    }

    fn get_result_from_cache(self, key: u8) -> (u8, u8) {
        let cache = self.cache.lock().unwrap();
        let key = CacheKey(key);
        let result = match cache.get(&key) {
            Some(result) => Some(result.clone()),
            None => None,
        };
        (key.0, result.unwrap()) 
    }


    fn insert_result_into_cache(self, key: u8) {
        let mut cache = self.cache.lock().unwrap();
        let cachekey = CacheKey(key.clone());
        cache.insert(cachekey, key);
    }

    fn get_cache_state(self, key: u8) -> (&'static Mutex<CacheState>, &'static Condvar) {
        let (lock, cvar) = {
            let mut state_map = self.cache_state.lock().unwrap();
            &*state_map
                .get_mut(&CacheKey(key))
                .expect("Entry in cache state to have been previously inserted")
                .clone()
        };
        (lock, cvar)
    }

}