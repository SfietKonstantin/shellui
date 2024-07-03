use clap::Command;
use colored::Colorize;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::borrow::Cow;

#[derive(Clone, Debug)]
enum CommandItem {
    Command(String),
    Arg(String),
}

#[derive(Clone, Debug)]
struct CommandLine(Vec<CommandItem>);

impl CommandLine {
    fn to_command_line_iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().filter_map(|item| {
            if let CommandItem::Command(command) = item {
                Some(command.as_str())
            } else {
                None
            }
        })
    }
}

pub struct Ui {
    commands: Vec<CommandLine>,
}

impl Ui {
    pub fn new(command: Command) -> Self {
        let commands = Self::parse_command_tree(&command);
        Ui { commands }
    }

    fn parse_command_tree(command: &Command) -> Vec<CommandLine> {
        let mut output = Vec::new();
        Self::recursive_fill_command_tree(command, Vec::new(), &mut output);
        output
    }

    fn recursive_fill_command_tree(
        parent: &Command,
        prefix: Vec<CommandItem>,
        output: &mut Vec<CommandLine>,
    ) {
        let mut help_line = prefix.clone();
        help_line.push(CommandItem::Command("help".to_string()));
        output.push(CommandLine(help_line));

        for command in parent.get_subcommands() {
            let mut line = prefix.clone();
            line.push(CommandItem::Command(command.get_name().to_string()));

            output.push(CommandLine(line.clone()));
            if command.has_subcommands() {
                Self::recursive_fill_command_tree(command, line, output);
            } else {
                for arg in command.get_positionals() {
                    line.push(CommandItem::Arg(arg.get_id().to_string()));
                    output.push(CommandLine(line.clone()));
                }
            }
        }
    }

    fn find_matches<'a, S>(
        &'a self,
        args: &'a [S],
        limit: usize,
    ) -> impl Iterator<Item = &'a CommandLine>
    where
        S: AsRef<str>,
    {
        self.commands
            .iter()
            .filter(move |command| {
                let command = command
                    .to_command_line_iter()
                    .take(limit)
                    .collect::<Vec<_>>();
                let args = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
                command == args
            })
            .filter(move |command| command.0.len() == limit.saturating_add(1))
    }

    fn find_matching_suggestions<'a, S>(
        &'a self,
        args: &'a [S],
        limit: usize,
        last_arg: &'a str,
    ) -> impl Iterator<Item = &'a str>
    where
        S: AsRef<str>,
    {
        self.find_matches(args, limit)
            .filter_map(move |command| command.to_command_line_iter().nth(limit))
            .filter(move |command| command.starts_with(last_arg))
    }

    fn solve_hint(&self, line: &str) -> Option<UiHint> {
        let args = shell_words::split(line).ok()?;
        let ends_with_whitespace = line.ends_with(char::is_whitespace);

        if ends_with_whitespace {
            // We want a suggestion of the next arg
            // but we will only suggest args
            let limit = args.len();

            let command = self.find_matches(&args, limit).next()?;

            let item = command.0.get(limit)?;
            if let CommandItem::Arg(name) = item {
                Some(UiHint(format!("<{name}>"), None))
            } else {
                None
            }
        } else {
            let limit = args.len().saturating_sub(1);
            let limited_args = args.iter().take(limit).collect::<Vec<_>>();
            let last_arg = args.last()?;
            let command = self
                .find_matching_suggestions(&limited_args, limit, last_arg.as_str())
                .next()?;

            let suffix = command.strip_prefix(last_arg)?;
            Some(UiHint(suffix.to_string(), Some(suffix.to_string())))
        }
    }

    fn solve_complete(&self, line: &str, pos: usize) -> Option<(usize, Vec<String>)> {
        let line = line.get(0..pos)?;
        let args = shell_words::split(line).ok()?;
        let ends_with_whitespace = line.ends_with(char::is_whitespace);

        if ends_with_whitespace || line.is_empty() {
            // We want completion of the next arg
            // and we will only complete with commands
            let limit = args.len();

            let completions = self
                .find_matches(&args, limit)
                .filter_map(|command| command.0.get(limit))
                .filter_map(|command| match command {
                    CommandItem::Command(name) => Some(name.clone()),
                    CommandItem::Arg(_) => None,
                })
                .collect();

            Some((line.len(), completions))
        } else {
            let last_arg = args.last()?;
            let index = line.rfind(last_arg)?;

            let limit = args.len().saturating_sub(1);
            let limited_args = args.iter().take(limit).collect::<Vec<_>>();
            let last_arg = args.last()?;
            let completions = self
                .find_matching_suggestions(&limited_args, limit, last_arg)
                .map(ToString::to_string)
                .collect();

            Some((index, completions))
        }
    }
}

impl Completer for Ui {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        Ok(self.solve_complete(line, pos).unwrap_or((pos, Vec::new())))
    }
}

impl Highlighter for Ui {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(hint.white().dimmed().to_string())
    }
}

impl Validator for Ui {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UiHint(String, Option<String>);

impl Hint for UiHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        self.1.as_deref()
    }
}

impl Hinter for Ui {
    type Hint = UiHint;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        self.solve_hint(line)
    }
}

impl Helper for Ui {}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Arg;

    #[test]
    fn test_solve_hint_partial() {
        let command = Command::new("test")
            .subcommand(Command::new("test1"))
            .subcommand(Command::new("test2"));
        let hint = Ui::new(command).solve_hint("te");
        assert_eq!(
            hint,
            Some(UiHint("st1".to_string(), Some("st1".to_string())))
        );
    }

    #[test]
    fn test_solve_hint_full() {
        let command = Command::new("test")
            .subcommand(Command::new("test1"))
            .subcommand(Command::new("test2"));
        let hint = Ui::new(command).solve_hint("test1");
        assert_eq!(hint, Some(UiHint("".to_string(), Some("".to_string()))));
    }

    #[test]
    fn test_solve_hint_partial_second() {
        let command = Command::new("test")
            .subcommand(
                Command::new("test1")
                    .subcommand(Command::new("test11"))
                    .subcommand(Command::new("test12")),
            )
            .subcommand(Command::new("test2"));
        let hint = Ui::new(command).solve_hint("test1 t");
        assert_eq!(
            hint,
            Some(UiHint("est11".to_string(), Some("est11".to_string())))
        );
    }

    #[test]
    fn test_solve_hint_no_match() {
        let command = Command::new("test")
            .subcommand(Command::new("test1"))
            .subcommand(Command::new("test2"));
        let hint = Ui::new(command).solve_hint("a");
        assert_eq!(hint, None);
    }

    #[test]
    fn test_solve_hint_args() {
        let command = Command::new("test")
            .subcommand(
                Command::new("test1")
                    .arg(Arg::new("arg1"))
                    .arg(Arg::new("arg2")),
            )
            .subcommand(Command::new("test2"));
        let hint = Ui::new(command).solve_hint("test1 ");
        assert_eq!(hint, Some(UiHint("<arg1>".to_string(), None)));
    }

    #[test]
    fn test_solve_complete_partial() {
        let command = Command::new("test")
            .subcommand(Command::new("test1"))
            .subcommand(Command::new("test2"));
        let complete = Ui::new(command).solve_complete("te", 1);
        assert_eq!(
            complete,
            Some((0, vec!["test1".to_string(), "test2".to_string()]))
        );
    }

    #[test]
    fn test_solve_complete_second() {
        let command = Command::new("test")
            .subcommand(
                Command::new("test1")
                    .subcommand(Command::new("test11"))
                    .subcommand(Command::new("test12")),
            )
            .subcommand(Command::new("test2"));
        let complete = Ui::new(command).solve_complete("test1 ", 6);
        assert_eq!(
            complete,
            Some((
                6,
                vec![
                    "help".to_string(),
                    "test11".to_string(),
                    "test12".to_string()
                ]
            ))
        );
    }
}
