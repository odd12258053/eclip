use eclip::{Application, Command};

#[derive(Command, Debug)]
#[allow(dead_code)]
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
    #[option(short = "v")]
    v: Vec<bool>,
    #[option(short = "o")]
    o: Option<i32>,
    #[option(short = "f")]
    f: f32,
}

impl Command for Command1 {
    fn run(&self) {
        println!("{:?}", self);
    }
}

fn main() {
    Application::new("ex_named")
        .add_command("", Command1::entry_point)
        .run();
}
