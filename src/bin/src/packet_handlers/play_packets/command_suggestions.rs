use std::sync::Arc;

use bevy_ecs::prelude::*;
use ferrumc_commands::{
    graph::node::CommandNodeType, infrastructure, Command, CommandContext, CommandInput, Sender,
    Suggestion, ROOT_COMMAND,
};
use ferrumc_net::{
    connection::StreamWriter,
    packets::outgoing::command_suggestions::{CommandSuggestionsPacket, Match},
    CommandSuggestionRequestReceiver,
};
use ferrumc_net_codec::net_types::{
    length_prefixed_vec::LengthPrefixedVec, prefixed_optional::PrefixedOptional, var_int::VarInt,
};
use ferrumc_state::GlobalStateResource;
use tracing::error;

fn find_command(input: String) -> Option<Arc<Command>> {
    let mut input = input;
    if input.starts_with("/") {
        input.remove(0);
    }

    if let Some(command) = ferrumc_commands::infrastructure::get_command_by_name(&input) {
        return Some(command);
    }

    if let Some(command) = ferrumc_commands::infrastructure::find_command(&input) {
        return Some(command);
    }

    while !input.is_empty() {
        // remove the last word and retry
        if let Some(pos) = input.rfind(char::is_whitespace) {
            input.truncate(pos);

            if let Some(command) = ferrumc_commands::infrastructure::get_command_by_name(&input) {
                return Some(command);
            }

            if let Some(command) = ferrumc_commands::infrastructure::find_command(&input) {
                return Some(command);
            }
        } else {
            break; // string does not have any further words, meaning it's just whitespace?
        }
    }

    None
}

fn create_ctx(input: String, command: Option<Arc<Command>>, sender: Sender) -> CommandContext {
    let input = input
        .strip_prefix(command.clone().map(|c| c.name).unwrap_or_default())
        .unwrap_or(&input)
        .trim_start();

    let input = CommandInput::of(input.to_string());
    CommandContext {
        input: input.clone(),
        command: command.unwrap_or(ROOT_COMMAND.clone()),
        sender,
    }
}

pub fn command_suggestions(input: String, sender: Sender) -> (Vec<Suggestion>, String) {
    if input.is_empty() {
        return (Vec::new(), String::new());
    }

    let command = find_command(input.clone());
    let command_arg = input
        .clone()
        .strip_prefix(&format!(
            "/{} ",
            command.clone().map(|c| c.name).unwrap_or_default()
        ))
        .unwrap_or(&input)
        .to_string();
    let mut ctx = create_ctx(command_arg.clone(), command.clone(), sender);
    let command_arg = command_arg.clone(); // ok borrow checker
    let tokens = command_arg.split(" ").collect::<Vec<&str>>();
    let Some(current_token) = tokens.last() else {
        // whitespace
        return (Vec::new(), String::new());
    };

    let mut suggestions = Vec::new();

    let graph = infrastructure::get_graph();
    if let Some(command) = command {
        let mut subcommands = Vec::new();
        graph.traverse(|node, _idx, _depth, parent| {
            if let Some(parent) = parent {
                if graph.get_command_name(parent) == command.name && node.node_type() == CommandNodeType::Literal {
                    subcommands.push(node.name.clone().unwrap());
                }
            }
        });
        if !subcommands.is_empty() {
            suggestions.extend(subcommands.into_iter().map(Suggestion::of).collect::<Vec<Suggestion>>());
        }

        for arg in command.args.clone() {
            let arg_suggestions = (arg.suggester)(&mut ctx);
            ctx.input.skip_whitespace(u32::MAX, true);
            if !ctx.input.has_remaining_input() {
                suggestions.extend(arg_suggestions);
                break;
            }
        }
    } else {
        // otherwise, we just add all commands to the suggestions list,
        // they're filtered on the input anyway
        let mut nodes = infrastructure::get_graph()
            .nodes
            .iter()
            .filter(|node| node.node_type() == CommandNodeType::Literal)
            .map(|node| node.name.clone().expect("impossible to get root node"))
            .collect::<Vec<String>>();

        nodes.sort_by(|a, b| a.len().cmp(&b.len()));
        suggestions = nodes.iter().map(Suggestion::of).collect();
    }

    (
        suggestions
            .into_iter()
            .filter(|sug| sug.content.starts_with(current_token))
            .collect(),
        current_token.to_string(),
    )
}

pub fn handle(
    receiver: Res<CommandSuggestionRequestReceiver>,
    query: Query<&StreamWriter>,
    state: Res<GlobalStateResource>,
) {
    for (request, entity) in receiver.0.try_iter() {
        if !state.0.players.is_connected(entity) {
            return;
        }

        let input = request.input;
        let (suggestions, current_token) =
            command_suggestions(input.clone(), Sender::Player(entity));
        let length = input.len();
        let start = length - current_token.len();

        if let Err(e) = query
            .get(entity)
            .unwrap()
            .send_packet(CommandSuggestionsPacket {
                transaction_id: request.transaction_id,
                matches: LengthPrefixedVec::new(
                    suggestions
                        .into_iter()
                        .map(|sug| Match {
                            content: sug.content,
                            tooltip: PrefixedOptional::new(sug.tooltip),
                        })
                        .collect(),
                ),
                length: VarInt::new(length as i32),
                start: VarInt::new(start as i32),
            })
        {
            error!("failed sending command suggestions to player: {e}")
        }
    }
}
