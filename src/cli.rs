use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Command {
    Post,
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "this is not a state machine, but a set of flags"
)]
#[derive(Parser, Debug, Default)]
#[command(
    about = concat!(env!("CARGO_CRATE_NAME"), " - list directory contents"), 
    disable_help_flag = true
)]
pub struct Flags {
    /// post to nostr
    #[arg(default_value = None)]
    pub command: Option<Command>,
}

impl Flags {
    /// Parse from `std::env::args_os()`, [exit][Error::exit] on error.
    // Wraps `clap::Parser` logic without direct trait imports
    // Equivalent to `Flags::parse()` here
    pub fn from_args() -> Self {
        Self::parse()
    }

    /// Check if the command is "post"
    pub fn post(&self) -> bool {
        matches!(self.command, Some(Command::Post))
    }
}
