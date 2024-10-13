use std::sync::{Arc, Mutex};
use crate::{CommandContext, CommandInput, ArgumentParser, ParserResult};

pub struct SingleStringParser;

impl<S> ArgumentParser<S> for SingleStringParser {
    fn parse(&self, _c: Arc<&CommandContext<S>>, input: Arc<Mutex<CommandInput>>) -> ParserResult {
        Ok(Box::new(input.lock().unwrap().read_string()))
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        SingleStringParser
    }
}

pub struct GreedyStringParser;

impl<S> ArgumentParser<S> for GreedyStringParser {
    fn parse(&self, _c: Arc<&CommandContext<S>>, input: Arc<Mutex<CommandInput>>) -> ParserResult {
        let mut result = String::new();

        loop {
            let token = input.lock().unwrap().read_string_skip_whitespace(false);

            if token.is_empty() {
                break;
            }

            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&token);
        }

        Ok(Box::new(result))
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        GreedyStringParser
    }
}