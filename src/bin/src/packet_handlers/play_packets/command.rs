use bevy_ecs::prelude::*;
use ferrumc_commands::{
    messages::CommandDispatched,
    Sender,
};
use ferrumc_net::ChatCommandPacketReceiver;

pub fn handle(
    receiver: Res<ChatCommandPacketReceiver>,
    mut dispatch_msgs: MessageWriter<CommandDispatched>,
) {
    for (event, entity) in receiver.0.try_iter() {
        let sender = Sender::Player(entity);
        dispatch_msgs.write(CommandDispatched {
            command: event.command.clone(),
            sender,
        });
    }
}
