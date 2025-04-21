use clap::{ArgAction, Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Command {
    Post,
    Fetch,
    Stream,
    Contacts
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "this is not a state machine, but a set of flags"
)]
#[derive(Parser, Debug, Default)]
#[command(
    about = concat!(env!("CARGO_CRATE_NAME"), " - minimalistic nostr client"), 
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

    /// Check if the command is "fetch"
    pub fn fetch(&self) -> bool {
        matches!(self.command, Some(Command::Fetch))
    }

    /// Check if the command is "stream"
    pub fn stream(&self) -> bool {
        matches!(self.command, Some(Command::Stream))
    }

    /// Check if the command is "contacts"
    pub fn contacts(&self) -> bool {
        matches!(self.command, Some(Command::Contacts))
    }
}
