use eclip::{Application, Command};

#[derive(Command, Debug)]
#[allow(dead_code)]
struct Command1(
    #[option(short = "q", long = "quite", default = false, help = "help message")] bool,
    #[option(help = "help message")] bool,
    #[argument(help = "integer1")] i32,
    #[argument] u32,
    #[option(short = "c", default = 10)] i32,
);

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
