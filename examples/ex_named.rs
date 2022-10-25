use eclip::{Application, Command};

#[derive(Command, Debug)]
struct Command1 {
    #[option(short = "q", long = "quite", default = false, help = "help message")]
    quite: bool,
    #[option(help = "help message")]
    other: bool,
    #[argument(help = "integer1")]
    a: i32,
    #[argument]
    b: u32,
    #[option(short = "c", default = 10)]
    c: i32,
}

impl Command for Command1 {
    fn run(&self) {
        println!("{:?}", self);
    }
}

fn main() {
    Application::new()
        .add_command("", Command1::entry_point)
        .run();
}
