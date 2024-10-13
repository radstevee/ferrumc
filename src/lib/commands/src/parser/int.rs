use std::sync::{Arc, Mutex};
use crate::{CommandContext, CommandInput, ArgumentParser, ParserResult};

pub struct IntParser;

impl<S> ArgumentParser<S> for IntParser {
    fn parse(&self, _c: Arc<&CommandContext<S>>, input: Arc<Mutex<CommandInput>>) -> ParserResult {
        let token = input.lock().unwrap().read_string();

        match token.parse::<u32>() {
            Ok(int) => Ok(Box::new(int)),
            Err(err) => Err(err.to_string())
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        IntParser
    }
}