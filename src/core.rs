use std::collections::BTreeMap;
use std::env;
use std::process::exit;

use crate::utils::help_message;
use crate::PADDING_SIZE;

pub trait ArgsNew {
    fn new(args: env::Args) -> Self;
}

pub trait Help {
    fn help(helper: Helper);
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
        println!(
            "Usage:\n  {} COMMAND [OPTIONS] [ARGS]...\nOptions:\n{}\nCommands:",
            helper.name,
            help_message(PADDING_SIZE),
        );
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
        println!(
            "Usage:\n  {} COMMAND [OPTIONS] [ARGS]...\nOptions:\n{}\nCommands:",
            helper.name,
            help_message(PADDING_SIZE),
        );
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
