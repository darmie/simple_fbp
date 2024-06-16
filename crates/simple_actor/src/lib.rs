extern crate serde_json;
extern crate rusty_pool;
pub mod actor;

mod test {
    use rusty_pool::ThreadPool;
    use serde_json::json;

    use crate::actor::{Actor, SimpleActor};

    #[test]
    fn test_actor() {
        let my_simple_actor = SimpleActor::new(|message, state, outport| {
            let (sender, _) = outport;
            sender.send(json!(format!(
                "{} world!",
                message.as_str().unwrap_or_default()
            )))?;
            Ok(())
        });

        my_simple_actor.spawn(&ThreadPool::default());

        let _ = my_simple_actor.get_inport().0.send(json!("Hello"));

        if let Ok(message) = my_simple_actor.get_outport().1.recv() {
            println!("{:?}", message);
            assert_eq!("Hello world!", message.as_str().unwrap_or_default());
        }
    }
}
