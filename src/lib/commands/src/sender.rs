//! Command senders.

use bevy_ecs::prelude::*;
use ferrumc_core::mq;
use ferrumc_text::TextComponent;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A possible command sender.
pub enum Sender {
    /// A player has sent a command.
    Player(Entity),

    /// The server console has sent a command.
    Server,
}

impl Sender {
    /// Sends the given `message` to this sender, and to the action bar
    /// if `actionbar` is true.
    pub fn send_message(&self, message: TextComponent, actionbar: bool) {
        match self {
            Sender::Player(entity) => mq::queue(message, actionbar, *entity),
            Sender::Server => {
                println!("{}", message.to_ansi_string()); // Tracing prints it as Debug meaning ANSI escape codes don't work
            }
        }
    }
}
