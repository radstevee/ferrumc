use bevy_ecs::prelude::{MessageReader, MessageWriter};
use ferrumc_commands::messages::{CommandDispatched, ResolvedCommandDispatched};
use ferrumc_commands::{infrastructure, Command, CommandContext, CommandInput, Sender};
use ferrumc_text::{NamedColor, TextComponent, TextComponentBuilder};
use std::sync::Arc;

fn resolve(
    input: String,
    sender: Sender,
) -> bevy_ecs::error::Result<(Arc<Command>, CommandContext), Box<TextComponent>> {
    let command = infrastructure::find_command(&input);
    if command.is_none() {
        return Err(Box::new(
            TextComponentBuilder::new(format!("Unknown command: {input}"))
                .color(NamedColor::Red)
                .build(),
        ));
    }

    let command = command.unwrap();
    let input = input
        .strip_prefix(command.name)
        .unwrap_or(&input)
        .trim_start();
    let input = CommandInput::of(input.to_string());
    let ctx = CommandContext {
        input: input.clone(),
        command: command.clone(),
        sender,
    };

    Ok((command, ctx))
}

pub fn command_resolver(
    mut reader: MessageReader<CommandDispatched>,
    mut writer: MessageWriter<ResolvedCommandDispatched>,
) {
    for dispatched in reader.read() {
        let sender = dispatched.sender;
        let resolved = resolve(dispatched.command.clone(), sender);

        match resolved {
            Err(err) => {
                sender.send_message(*err, false);
            }

            Ok((command, ctx)) => {
                writer.write(ResolvedCommandDispatched {
                    command,
                    ctx,
                    sender,
                });
            }
        }
    }
}
