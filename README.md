# eclip

Eclip is a library for building CLI applications.

## For example
```rust
use eclip::{Application, Command, SubCommand};

#[derive(Command, Debug)]
struct Command1 {
    #[option(short = "q")]
    quite: bool,
    #[argument]
    a: i32,
    #[argument]
    b: u32,
}

impl Command for Command1 {
    fn run(&self) {
        println!("Run Command 1: {:?}", self)
    }
}

#[derive(Command, Debug)]
struct Command2 {}

impl Command for Command2 {
    fn run(&self) {
        println!("Run Command 2")
    }
}

#[derive(Command, Debug)]
struct Command3;

impl Command for Command3 {
    fn run(&self) {
        println!("Run Command 3")
    }
}

#[derive(Command, Debug)]
struct Command4 ();

impl Command for Command4 {
    fn run(&self) {
        println!("Run Command 4")
    }
}

fn main() {
    Application::new("ex")
        .add_command("cmd1", Command1::entry_point)
        .add_command("cmd2", Command2::entry_point)
        .add_subcommand(
            "sub",
            SubCommand::new()
                .add_command("cmd3", Command3::entry_point)
                .add_command("cmd4", Command4::entry_point),
        )
        .run();
}
```
