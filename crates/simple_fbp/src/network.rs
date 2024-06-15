use std::{any, collections::HashMap, thread};

use rusty_pool::ThreadPool;
use simple_actor::actor::{Actor, Message, SimpleActor};

pub struct Connection {
    from: String,
    to: String,
}

pub struct Network {
    actors: HashMap<String, SimpleActor>,
    connections: Vec<Connection>,
    thread_pool: ThreadPool
}

impl Network {
    pub fn new() -> Network {
        Network {
            actors: HashMap::new(),
            connections: Vec::new(),
            thread_pool: ThreadPool::default()
        }
    }
    pub fn add_actor(&mut self, id: &str, actor: SimpleActor) {
        self.actors.insert(id.to_owned(), actor);
    }

    pub fn add_connection(&mut self, from_id: &str, to_id: &str) {
        self.connections.push(Connection {
            from: from_id.to_owned(),
            to: to_id.to_owned(),
        });
    }

    pub fn trigger(&self, id: &str, message: Message) -> Result<(), anyhow::Error> {
        if let Some(actor) = self.actors.get(id) {
            let (sender, _) = actor.get_inport();
            sender.send(message)?;
        }

        Ok(())
    }

    pub fn start(&self) {
        for connection in &self.connections {
            let actor_1 = self
                .actors
                .get(&connection.from)
                .expect("expected actor to be found");
            let actor_2 = self
                .actors
                .get(&connection.to)
                .expect("expected actor to be found");

            let (_, reciever) = actor_1.get_outport().clone();
            let (sender, _) = actor_2.get_inport().clone();

            self.thread_pool.execute(move || loop {
                if let Ok(message) = reciever.recv() {
                    let _ = sender.send(message);
                }
            });
        }
    }
}

mod test {

    use serde_json::json;
    use simple_actor::actor::SimpleActor;

    use super::Network;

    #[test]
    fn test_network() {
        let mut network = Network::new();

        let notify = SimpleActor::new(|message, _, outport| {
            let (sender, _) = outport;
            sender.send(json!(format!(
                "{} world!",
                message.as_str().unwrap_or_default()
            )))?;
            Ok(())
        });

        let logger = SimpleActor::new(|message, _, _outport| {
            println!("[LOG] {}", message.as_str().unwrap_or_default());
            Ok(())
        });

        network.add_actor("notify", notify);
        network.add_actor("logger", logger);

        // Start all actors
        for (_, actor) in &network.actors {
            actor.spawn(&network.thread_pool)
        }

        network.add_connection("notify", "logger");

        network.start();

        let _ = network.trigger("notify", json!("Hello"));

        loop {}
    }
}
