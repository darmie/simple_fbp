use std::{collections::HashMap, sync::{Arc, Mutex}, thread::{self, Thread}};

use rusty_pool::ThreadPool;
use serde_json::{json, Value};

pub type Channel<T> = (flume::Sender<T>, flume::Receiver<T>);

pub type Message = Value;

pub type Behavior = dyn (FnMut(Message, Arc<Mutex<HashMap<String, Value>>>, Channel<Message>) -> Result<(), anyhow::Error>) + Send + Sync + 'static;

pub trait Actor {
    fn get_inport(&self) -> &Channel<Message>;
    fn get_outport(&self) -> &Channel<Message>;
}

pub struct SimpleActor {
    inport: Channel<Message>,
    outport: Channel<Message>,
    state: Arc<Mutex<HashMap<String, Value>>>,
    behavior:Arc<Mutex<Behavior>>
}

impl Actor for SimpleActor {
    fn get_inport(&self) -> &Channel<Message> {
        &self.inport
    }

    fn get_outport(&self) -> &Channel<Message> {
        &self.outport
    }
}

impl SimpleActor {
    pub fn new(behavior: impl (FnMut(Message, Arc<Mutex<HashMap<String, Value>>>, Channel<Message>) -> Result<(), anyhow::Error>) + Send + Sync + 'static) -> SimpleActor {
        Self {
            inport: flume::bounded(10),
            outport: flume::unbounded(),
            state: Arc::new(Mutex::new(HashMap::new())),
            behavior: Arc::new(Mutex::new(behavior))
        }
    }
    pub fn spawn(&self, thread_pool:&ThreadPool) {
        let _inport = self.inport.clone();
        let _outport = self.outport.clone();
        let _behavior = self.behavior.clone();
        let _state = self.state.clone();


        // Run behavior
        thread_pool.execute(move || {
            if let Ok(ref mut behavior) = _behavior.clone().lock() {
                loop {
                    let (_, reciever) = _inport.clone();
                    if let Ok(message) = reciever.try_recv() {
                        let _ = (behavior)(message, _state.clone(), _outport.clone());
                    }
                }
            }
        });
    }
}
