mod ui;

use self::ui::ShellUi;
use crate::errors::DisplayCli;
use crate::{Context, ShellParser};
use clap::{CommandFactory, Parser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{CompletionType, Config, Editor};
use std::io::{Error, ErrorKind, Result};
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
    pub fn try_run(context: &mut T::Context, line: &str) -> Result<ShellAction> {
        let parsed = shell_words::split(line).map_err(Error::other)?;

        if !parsed.is_empty() {
            let iter = iter::once("shellui").chain(parsed.iter().map(String::as_str));
            match ShellArgs::<T>::try_parse_from(iter) {
                Ok(args) => args.command.run(context),
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
    fn run(&self, context: &mut T::Context) -> Result<ShellAction> {
        match self {
            ShellCommand::Common(command) => match T::run_command(context, command) {
                Ok(()) => Ok(ShellAction::None),
                Err(error) => match error.kind() {
                    ErrorKind::Interrupted => Ok(ShellAction::None),
                    _ => {
                        error.display_cli();
                        Ok(ShellAction::None)
                    }
                },
            },
            ShellCommand::Clear => Ok(ShellAction::ClearScreen),
            ShellCommand::Exit => Ok(ShellAction::Eof),
        }
    }
}

pub fn launch_shell<T>(context: &mut T::Context) -> Result<()>
where
    T: ShellParser,
{
    let history_path = context.history_path();
    let helper = ShellUi::new(ShellArgs::<T>::command());
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .auto_add_history(true)
        .build();
    let mut rl: Editor<ShellUi, FileHistory> = Editor::with_config(config).map_err(Error::other)?;
    rl.set_helper(Some(helper));
    if let Some(history_path) = &history_path {
        rl.load_history(&history_path).map_err(Error::other)?;
    }

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => match ShellArgs::<T>::try_run(context, &line)? {
                ShellAction::None => {}
                ShellAction::ClearScreen => rl.clear_screen().map_err(Error::other)?,
                ShellAction::Eof => break,
            },
            Err(ReadlineError::Interrupted) => {
                // Continue
            }
            Err(ReadlineError::Eof) => break,
            Err(_) => break,
        }
    }

    if let Some(history_path) = history_path {
        rl.save_history(&history_path).map_err(Error::other)?;
    }

    Ok(())
}
