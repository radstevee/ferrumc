use std::{any::Any, future::Future, pin::Pin, sync::Arc};
use std::sync::Mutex;

pub mod parser;

pub struct CommandContext<S> {
    pub input: Arc<Mutex<CommandInput>>,
    pub command: Arc<Command<S>>,
    pub state: S,
}

impl<S> CommandContext<S> {
    pub fn new(input: CommandInput, command: Arc<Command<S>>, state: S) -> Arc<Self> {
        Arc::new(Self {
            input: Arc::new(Mutex::new(input)),
            command,
            state,
        })
    }
}

pub struct CommandArgument<S> {
    pub name: String,
    pub required: bool,
    pub parser: Box<dyn ArgumentParser<S>>,
}

impl<S> CommandArgument<S> {
    pub fn new(name: String, required: bool, parser: Box<dyn ArgumentParser<S>>) -> Self {
        CommandArgument {
            name,
            required,
            parser,
        }
    }
}

pub type ParserResult = Result<Box<dyn Any>, String>;
pub type CommandResult = Result<String, String>; // TODO: replace with something more sensible, text component impl needed
pub type CommandOutput = Pin<Box<dyn Future<Output=CommandResult> + Send + 'static>>;

pub struct Command<S> {
    pub name: String,
    pub args: Vec<CommandArgument<S>>,
    pub executor: Arc<dyn for<'a> Fn(Arc<CommandContext<S>>) -> CommandOutput + Send + Sync + 'static>,
}

impl<S> Command<S> {
    pub fn execute(&self, c: Arc<CommandContext<S>>) -> CommandOutput {
        (self.executor)(c)
    }

    pub fn validate(&self, c: Arc<&CommandContext<S>>, input: Arc<Mutex<CommandInput>>) -> Result<(), String> {
        for arg in &self.args {
            arg.parser.parse(c.clone(), input.clone())?;
        }

        Ok(())
    }
}

impl<S> CommandContext<S> {
    pub fn arg<T: Any>(&self, name: &str) -> Result<T, String> {
        if let Some(arg) = self.command.args.iter().find(|a| a.name == name) {
            let input = self.input.clone();
            let result = arg.parser.parse(Arc::new(self), input);

            return match result {
                Ok(b) => match b.downcast::<T>() {
                    Ok(value) => Ok(*value),
                    Err(_) => Err("Failed to downcast".to_string()),
                },
                Err(err) => Err(err),
            };
        } else {
            Err(format!("Argument '{}' not found.", name))
        }
    }
}

pub trait ArgumentParser<S>: Send + Sync {
    fn parse(&self, context: Arc<&CommandContext<S>>, input: Arc<Mutex<CommandInput>>) -> ParserResult;
    fn new() -> Self
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct CommandInput {
    pub input: String,
    pub cursor: u32,
}

impl CommandInput {
    pub fn of(string: String) -> Self {
        Self {
            input: string,
            cursor: 0,
        }
    }
    pub fn append_string(&mut self, string: String) {
        self.input += &*string;
    }
    pub fn move_cursor(&mut self, chars: u32) {
        if self.cursor + chars > self.input.len() as u32 {
            return;
        }

        self.cursor += chars;
    }
    pub fn remaining_length(&self) -> u32 {
        self.input.len() as u32 - self.cursor
    }
    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.cursor as usize)
    }
    pub fn has_remaining_input(&self) -> bool {
        self.cursor < self.input.len() as u32
    }
    pub fn skip_whitespace(&mut self, max_spaces: u32, preserve_single: bool) {
        if preserve_single && self.remaining_length() == 1 && self.peek() == Some(' ') {
            return;
        }

        let mut i = 0;
        while i < max_spaces
            && self.has_remaining_input()
            && self.peek().map_or(false, |c| c.is_whitespace())
        {
            self.read(1);
            i += 1;
        }
    }
    pub fn remaining_input(&self) -> String {
        self.input[..self.cursor as usize].to_string()
    }
    pub fn peek_string_chars(&self, chars: u32) -> String {
        let remaining = self.remaining_input();
        if chars > remaining.len() as u32 {
            return "".to_string();
        }

        remaining[0..chars as usize].to_string()
    }
    pub fn read(&mut self, chars: u32) -> String {
        let read_string = self.peek_string_chars(chars);
        self.move_cursor(chars);
        read_string
    }
    pub fn remaining_tokens(&self) -> u32 {
        let count = self.remaining_input().split(' ').count() as u32;
        if self.remaining_input().ends_with(' ') {
            return count + 1;
        }
        count
    }
    pub fn read_string(&mut self) -> String {
        self.skip_whitespace(u32::MAX, false);
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                break;
            }
            result.push(c);
            self.move_cursor(1);
        }
        result
    }
    pub fn peek_string(&self) -> String {
        let remaining = self.remaining_input();
        remaining
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }
    pub fn read_until(&mut self, separator: char) -> String {
        self.skip_whitespace(u32::MAX, false);
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if c == separator {
                self.move_cursor(1);
                break;
            }
            result.push(c);
            self.move_cursor(1);
        }
        result
    }
    pub fn read_string_skip_whitespace(&mut self, preserve_single: bool) -> String {
        let read_string = self.read_string();
        self.skip_whitespace(u32::MAX, preserve_single);
        read_string
    }
}