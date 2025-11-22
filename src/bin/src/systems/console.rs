use std::{
    io::{stdout, Write},
    time::Duration,
};

use bevy_ecs::{message::MessageWriter, resource::Resource, system::ResMut};
use crossterm::{
    cursor::MoveToColumn,
    event,
    style::{ResetColor, SetForegroundColor},
    terminal::{BeginSynchronizedUpdate, Clear, EndSynchronizedUpdate},
    ExecutableCommand,
};
use ferrumc_commands::{messages::CommandDispatched, Sender};

use crate::packet_handlers::play_packets::command_suggestions;

#[derive(Resource, Default)]
pub struct CurrentInput(pub String);

fn suggestion(input: &str) -> Option<String> {
    let (suggestions, _) =
        command_suggestions::command_suggestions(input.to_string(), Sender::Server);
    suggestions.get(0).map(|s| s.content.clone())
}

fn redraw(input: &str) {
    let sugg = suggestion(input);
    let mut stdout = stdout();

    let _ = stdout.execute(BeginSynchronizedUpdate);
    let _ = stdout.execute(Clear(crossterm::terminal::ClearType::CurrentLine));
    let _ = stdout.execute(MoveToColumn(0));

    print!("> {}", input);

    if let Some(s) = sugg {
        let remaining = &s[input.len()..];
        let _ = stdout.execute(SetForegroundColor(crossterm::style::Color::DarkGrey));
        print!("{remaining}");
        let _ = stdout.execute(ResetColor);
    }

    // move cursor back to the position *after the real input*
    let cursor_x = 2 + input.len() as u16; // "> " = 2 chars
    let _ = stdout.execute(MoveToColumn(cursor_x));

    let _ = stdout.execute(EndSynchronizedUpdate);
    stdout.flush().expect("stdout flush failed");
}

pub fn console(mut writer: MessageWriter<CommandDispatched>, mut input: ResMut<CurrentInput>) {
    let command = &mut input.0;

    if !event::poll(Duration::from_millis(0)).unwrap_or(false) {
        return;
    }

    if let Ok(event::Event::Key(key)) = event::read() {
        match key.code {
            event::KeyCode::Enter => {
                writer.write(CommandDispatched {
                    command: command.clone(),
                    sender: Sender::Server,
                });
                command.clear();
            }
            event::KeyCode::Char(c) => {
                command.push(c);
            }
            event::KeyCode::Backspace => {
                command.pop();
            }
            event::KeyCode::Tab | event::KeyCode::Right => {
                if let Some(s) = suggestion(command) {
                    *command = s;
                }
            }
            _ => {}
        }

        redraw(command);
    }
}
