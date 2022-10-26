//! Eclip is a library for building CLI applications.
//! # Example
//! ```
//! use eclip::{Application, Command, SubCommand};
//!
//! #[derive(Command, Debug)]
//! struct Command1 {
//!     #[option(short="q")]
//!     quite: bool,
//!     #[argument]
//!     a: i32,
//!     #[argument]
//!     b: u32,
//! }
//!
//! impl Command for Command1 {
//!     fn run(&self) {
//!         println!("Run Command 1: {:?}", self)
//!     }
//! }
//!
//! #[derive(Command, Debug)]
//! struct Command2 {}
//!
//! impl Command for Command2 {
//!     fn run(&self) {
//!         println!("Run Command 2")
//!     }
//! }
//!
//! #[derive(Command, Debug)]
//! struct Command3;
//!
//! impl Command for Command3 {
//!     fn run(&self) {
//!         println!("Run Command 3")
//!     }
//! }
//!
//! #[derive(Command, Debug)]
//! struct Command4 ();
//!
//! impl Command for Command4 {
//!     fn run(&self) {
//!         println!("Run Command 4")
//!     }
//! }
//!
//! Application::new()
//!     .add_command("cmd1", Command1::entry_point)
//!     .add_command("cmd2", Command2::entry_point)
//!     .add_subcommand(
//!         "sub",
//!         SubCommand::new()
//!             .add_command("cmd3", Command3::entry_point)
//!             .add_command("cmd4", Command4::entry_point),
//!     )
//!     .run();
//! ```

use std::collections::BTreeMap;
use std::env;
use std::process::exit;

pub use eclip_derive::Command;

pub const PADDING_SIZE: usize = 30;

enum Runner<'a> {
    FType(fn(Helper)),
    MType(SubCommand<'a>),
}

pub struct Helper<'a> {
    pub args: env::Args,
    pub help: bool,
    pub name: &'a str,
    pub cmds: Vec<String>,
    pub version: &'a str,
}

pub fn help_message(padding: usize) -> String {
    format!(
        "  {:<padding$} {}\n  {:<padding$} {}",
        "--help", "Show this message.", "--version", "Show this version.",
    )
}

impl<'a> Helper<'a> {
    pub fn new(args: env::Args, help: bool) -> Self {
        Self {
            args,
            help,
            name: option_env!("CARGO_BIN_NAME")
                .or(option_env!("CARGO_PKG_NAME"))
                .unwrap_or_default(),
            cmds: Vec::new(),
            version: option_env!("CARGO_PKG_VERSION").unwrap_or_default(),
        }
    }

    pub fn command(&self) -> String {
        let mut command = self.name.to_string();
        for cmd in &self.cmds {
            command.push_str(&format!(" {}", cmd));
        }
        command
    }
}

pub struct Application<'a> {
    cmds: BTreeMap<&'a str, Runner<'a>>,
}

impl<'a> Application<'a> {
    pub fn new() -> Self {
        Self {
            cmds: BTreeMap::new(),
        }
    }

    pub fn add_command(mut self, name: &'a str, cmd: fn(Helper)) -> Self {
        self.cmds.insert(name, Runner::FType(cmd));
        self
    }

    pub fn add_subcommand(mut self, name: &'a str, subcmd: SubCommand<'a>) -> Self {
        self.cmds.insert(name, Runner::MType(subcmd));
        self
    }

    fn help(&self, helper: Helper) {
        println!("Usage:\n  {} COMMAND [OPTIONS] [ARGS]...", helper.name);
        println!("\nOptions:\n{}", help_message(PADDING_SIZE));
        println!("\nCommands:");
        for cmd in self.cmds.keys() {
            println!("  {}", cmd)
        }
    }

    pub fn run(&self) {
        let mut helper = Helper::new(env::args(), false);

        if env::args().any(|arg| &arg == "--version") {
            println!("{}", helper.version);
            exit(0);
        }
        helper.help = env::args().any(|arg| &arg == "--help");
        let _process = helper.args.next();

        if self.cmds.len() == 1 {
            let key = self.cmds.keys().next().unwrap();
            match self.cmds.get(key).unwrap() {
                Runner::FType(cmd) => cmd(helper),
                Runner::MType(cmd) => cmd.run(helper),
            }
        } else {
            match helper.args.next() {
                Some(cmd) => match self.cmds.get(cmd.as_str()) {
                    Some(runner) => {
                        helper.cmds.push(cmd);
                        match runner {
                            Runner::FType(cmd) => cmd(helper),
                            Runner::MType(cmd) => cmd.run(helper),
                        }
                    }
                    _ => self.help(helper),
                },
                _ => self.help(helper),
            }
        }
    }
}

impl<'a> Default for Application<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ArgsNew {
    fn new(args: env::Args) -> Self;
}

pub trait Help {
    fn help(_helper: Helper);
}

pub trait Command {
    fn run(&self);
    fn entry_point(helper: Helper)
    where
        Self: Sized + ArgsNew,
        Self: Help,
    {
        if helper.help {
            <Self as Command>::help(helper);
        } else {
            Self::new(helper.args).run();
        }
    }
    fn help(helper: Helper)
    where
        Self: Help,
    {
        <Self as Help>::help(helper);
    }
}

pub struct SubCommand<'a> {
    cmds: BTreeMap<&'a str, Runner<'a>>,
}

impl<'a> SubCommand<'a> {
    pub fn new() -> Self {
        Self {
            cmds: BTreeMap::new(),
        }
    }

    fn help(&self, helper: Helper) {
        println!("Usage:\n  {} COMMAND [OPTIONS] [ARGS]...", helper.command());
        println!("\nOptions:\n{}", help_message(PADDING_SIZE));
        println!("\nCommands:");
        for cmd in self.cmds.keys() {
            println!("  {}", cmd)
        }
    }

    pub fn add_command(mut self, name: &'a str, cmd: fn(Helper)) -> Self {
        self.cmds.insert(name, Runner::FType(cmd));
        self
    }

    pub fn add_subcommand(mut self, name: &'a str, subcmd: SubCommand<'a>) -> Self {
        self.cmds.insert(name, Runner::MType(subcmd));
        self
    }

    fn run(&self, mut helper: Helper) {
        if self.cmds.len() == 1 {
            let key = self.cmds.keys().next().unwrap();
            match self.cmds.get(key).unwrap() {
                Runner::FType(cmd) => cmd(helper),
                Runner::MType(cmd) => cmd.run(helper),
            }
        } else {
            match helper.args.next() {
                Some(cmd) => match self.cmds.get(cmd.as_str()) {
                    Some(runner) => {
                        helper.cmds.push(cmd);
                        match runner {
                            Runner::FType(cmd) => cmd(helper),
                            Runner::MType(cmd) => cmd.run(helper),
                        }
                    }
                    _ => self.help(helper),
                },
                _ => self.help(helper),
            }
        }
    }
}

impl<'a> Default for SubCommand<'a> {
    fn default() -> Self {
        Self::new()
    }
}
