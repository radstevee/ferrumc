use std::{borrow::Cow, sync::LazyLock, thread};

use bevy_ecs::message::MessageWriter;
use crossbeam_queue::SegQueue;
use ferrumc_commands::{Sender, messages::CommandDispatched};
use ferrumc_state::GlobalState;
use ferrumc_text::{Color, NamedColor};
use rustyline::{Completer, Editor, Helper, Validator, highlight::Highlighter, hint::Hinter};
use tracing::info;

use crate::packet_handlers::play_packets::command_suggestions;

static QUEUE: LazyLock<crossbeam_queue::SegQueue<String>> = LazyLock::new(SegQueue::new);

fn suggestion(input: &str) -> Option<(String, String)> {
    let (suggestions, tok) =
        command_suggestions::command_suggestions(input.to_string(), Sender::Server);
    suggestions.get(0).map(|s| (s.content.clone(), tok))
}

#[derive(Completer, Helper, Validator)]
struct ConsoleHelper;

impl Highlighter for ConsoleHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[0m{}{hint}\x1b[0m", Color::Named(NamedColor::DarkGray).to_ansi_color().expect("dark gray should have an ansi color")))
    }
}

impl Hinter for ConsoleHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let (suggestion, token) = suggestion(line)?;

        Some(suggestion.strip_prefix(&token).unwrap_or(&suggestion).to_string())
    }
}

pub fn init_console(state: GlobalState) {
    thread::spawn(move || {
        let mut line = Editor::new().unwrap();
        line.set_helper(Some(ConsoleHelper));

        loop {
            if let Ok(command) = line.readline("> ") {
                QUEUE.push(command);
            } else {
                info!("Shutting down server...");
                state
                    .shut_down
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                state
                    .world
                    .sync()
                    .expect("Failed to sync world before shutdown");
                break
            }
        }
    });
}

pub fn console_sender(mut writer: MessageWriter<CommandDispatched>) {
    while !QUEUE.is_empty() {
        let entry = QUEUE.pop().unwrap();
        writer.write(CommandDispatched { command: entry, sender: Sender::Server });
    }
}
