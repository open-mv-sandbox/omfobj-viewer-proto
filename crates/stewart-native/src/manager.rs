use anyhow::Error;
use stewart::{handler::Handler, ActorOps};
use stewart_messages::StartActor;

pub struct StartActorHandler;

impl Handler for StartActorHandler {
    type Message = StartActor;

    fn handle(&self, ops: &dyn ActorOps, message: Self::Message) -> Result<(), Error> {
        // TODO: Actually manage actors, this just runs the handlers in-line
        // TODO: Do something with errors
        message.run_factory(ops)?;

        Ok(())
    }
}
