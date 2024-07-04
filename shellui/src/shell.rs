mod ui;

use self::ui::ShellUi;
use crate::errors::IoErrorExt;
use crate::ShellParser;
use clap::{CommandFactory, Parser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{CompletionType, Config, Editor};
use std::io::{Error, Result};
use std::iter;

#[derive(Parser)]
#[command(bin_name = "", disable_version_flag = true, disable_help_flag = true)]
struct ShellArgs<T>
where
    T: ShellParser,
{
    #[command(subcommand)]
    command: ShellCommand<T>,
}
impl<T> ShellArgs<T>
where
    T: ShellParser,
{
    pub fn try_run(line: &str) -> Result<ShellAction> {
        let parsed = shell_words::split(line).map_err(Error::other)?;

        if !parsed.is_empty() {
            let iter = iter::once("shellui").chain(parsed.iter().map(String::as_str));
            match ShellArgs::<T>::try_parse_from(iter) {
                Ok(args) => args.command.run(),
                Err(error) => {
                    error.print()?;
                    Ok(ShellAction::None)
                }
            }
        } else {
            Ok(ShellAction::None)
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
enum ShellCommand<T>
where
    T: ShellParser,
{
    #[command(flatten)]
    Common(T::Commands),
    /// Clear the shell
    Clear,
    /// Exit the shell
    Exit,
}

pub enum ShellAction {
    None,
    ClearScreen,
    Eof,
}

impl<T> ShellCommand<T>
where
    T: ShellParser,
{
    fn run(&self) -> Result<ShellAction> {
        match self {
            ShellCommand::Common(command) => match T::run_command(command) {
                Ok(()) => Ok(ShellAction::None),
                Err(error) => {
                    error.display_cli();
                    Err(error)
                }
            },
            ShellCommand::Clear => Ok(ShellAction::ClearScreen),
            ShellCommand::Exit => Ok(ShellAction::Eof),
        }
    }
}

pub fn launch_shell<T>() -> Result<()>
where
    T: ShellParser,
{
    let helper = ShellUi::new(ShellArgs::<T>::command());
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl: Editor<ShellUi, FileHistory> = Editor::with_config(config).map_err(Error::other)?;
    rl.set_helper(Some(helper));

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                match ShellArgs::<T>::try_run(&line)? {
                    ShellAction::None => {}
                    ShellAction::ClearScreen => rl.clear_screen().map_err(Error::other)?,
                    ShellAction::Eof => break,
                }
                rl.add_history_entry(line.as_str()).map_err(Error::other)?;
            }
            Err(ReadlineError::Interrupted) => {
                // Continue
            }
            Err(ReadlineError::Eof) => break,
            Err(_) => break,
        }
    }

    Ok(())
}
