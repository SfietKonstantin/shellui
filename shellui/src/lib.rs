pub mod errors;
pub mod format;
pub mod input;
mod shell;

use crate::errors::{ShellUiError, ShellUiResult};
use crate::format::AsFormatted;
use clap::{Parser, Subcommand};
use std::io::Result;
use std::path::PathBuf;
use std::process::exit;

/// Shell context
pub trait Context: Sized {
    fn new() -> Result<Self>;
    fn history_path(&self) -> Option<PathBuf>;
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
    fn run_command(context: &mut Self::Context, command: &Self::Commands) -> ShellUiResult<()>;
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
        match error {
            ShellUiError::Error(_) | ShellUiError::Warning(_) => error.print_formatted(),
            ShellUiError::Interrupt => {}
        }
        exit(1);
    }
}

fn handle_launch<T>() -> ShellUiResult<()>
where
    T: ShellParser,
{
    let mut context = T::Context::new()?;
    let args = T::parse();
    if let Some(commands) = args.try_get_command() {
        T::run_command(&mut context, &commands)
    } else {
        shell::launch_shell::<T>(&mut context)?;
        Ok(())
    }
}
