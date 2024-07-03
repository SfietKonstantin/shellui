mod ui;

use self::ui::Ui;
use clap::{CommandFactory, Parser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{CompletionType, Config, Editor, Result};
use std::{io, iter};

pub trait ShellParser: Parser {
    type Commands: Commands;
    fn try_get_command(self) -> Option<Self::Commands>;
}

pub trait Commands: Subcommand {
    fn run(&self) -> io::Result<()>;
}

pub fn launch<T>() -> Result<()>
where
    T: ShellParser,
{
    let args = T::parse();
    if let Some(commands) = args.try_get_command() {
        commands.run()?;
        Ok(())
    } else {
        launch_shell::<T::Commands>()
    }
}

#[derive(Debug, Parser)]
#[command(bin_name = "", disable_version_flag = true, disable_help_flag = true)]
pub struct ShellArgs<T>
where
    T: Commands,
{
    #[command(subcommand)]
    command: ShellCommand<T>,
}
impl<T> ShellArgs<T>
where
    T: Commands,
{
    pub fn try_run(line: &str) -> io::Result<ShellAction> {
        let parsed = shell_words::split(line).map_err(io::Error::other)?;

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
    T: Commands,
{
    #[command(flatten)]
    Common(T),
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
    T: Commands,
{
    fn run(&self) -> io::Result<ShellAction> {
        match self {
            ShellCommand::Common(command) => {
                command.run()?;
                Ok(ShellAction::None)
            }
            ShellCommand::Clear => Ok(ShellAction::ClearScreen),
            ShellCommand::Exit => Ok(ShellAction::Eof),
        }
    }
}

fn launch_shell<T>() -> Result<()>
where
    T: Commands,
{
    let helper = Ui::new(ShellArgs::<T>::command());
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl: Editor<Ui, FileHistory> = Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                match ShellArgs::<T>::try_run(&line)? {
                    ShellAction::None => {}
                    ShellAction::ClearScreen => rl.clear_screen()?,
                    ShellAction::Eof => break,
                }
                rl.add_history_entry(line.as_str())?;
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
