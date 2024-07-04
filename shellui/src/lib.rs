pub mod errors;
pub mod input;
mod shell;

use crate::errors::DisplayCli;
use clap::{Parser, Subcommand};
use std::io::{ErrorKind, Result};
use std::process::exit;

/// Shell context
pub trait Context: Sized {
    fn new() -> Result<Self>;
}

/// Clap extension to enable shell
///
/// This trait extension extends clap's `Parser` to enable
/// shell support.
///
/// Shellui uses clap subcommands as shell commands, but also
/// supports acting like a CLI. Implement a clap main entry point
/// that optionally takes subcommands to either process the subcommand
/// or enter in the shell.
pub trait ShellParser: Parser {
    /// Context
    type Context: Context;
    /// Subcommands
    type Commands: Subcommand;
    /// Try get command
    ///
    /// The clap main entrypoint should contain an optional subcommand,
    /// so that it can go into shell mode if the subcommand is not passed.
    fn try_get_command(self) -> Option<Self::Commands>;
    /// Run a command
    fn run_command(context: &mut Self::Context, command: &Self::Commands) -> Result<()>;
}

/// Launch a command
///
/// Will launch the entrypoint being passed, either running as a CLI
/// or spawning a shell.
pub fn launch<T>()
where
    T: ShellParser,
{
    if let Err(error) = handle_launch::<T>() {
        error.display_cli();
        exit(1);
    }
}

fn handle_launch<T>() -> Result<()>
where
    T: ShellParser,
{
    let mut context = T::Context::new()?;
    let args = T::parse();
    if let Some(commands) = args.try_get_command() {
        match T::run_command(&mut context, &commands) {
            Ok(()) => Ok(()),
            Err(error) => match error.kind() {
                ErrorKind::Interrupted => Ok(()),
                _ => Err(error),
            },
        }
    } else {
        shell::launch_shell::<T>(&mut context)
    }
}
