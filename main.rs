extern crate getopts;
extern crate yaml_rust;

use getopts::Options;
use getopts::Matches;
use std::env;
use std::process::exit;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/rails_app.rs"));

fn do_work(inp: &str, out: Option<String>) {
    println!("{}", inp);
    match out {
        Some(x) => println!("{}", x),
        None => println!("No Output"),
    }
}

struct CommandFactory {
}

impl CommandFactory {
    fn create(app_root: String) {
    }
}

fn usage(program: &str, opts: Options) -> String{
    let brief = format!("Usage: {} [options] COMMAND", program);
    return opts.usage(&brief) + "
Commands:
    digest         Prints the digest of the rails application

    command list   Lists all commands
    command info   Prints info on <cmd> command

    image list     List all image
    image info     Prints info on <img> image
    image label    Prints the docker label of specifed image
    image build    Builds specified docker image
    image push     Pushes specified docker image
    image pull     Pulls specified docker image
    image clean    Cleans working data for building specifed docker image
";
}

fn parse_arguments(args: Vec<String>) -> Result<Matches, String> {
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "", "Location of Application. Defaults to '.'", "APP");
    opts.optopt("i", "", "Name of image. Defaults to devbase", "IMAGE");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { return Err(f.to_string()) }
    };
    if matches.opt_present("h") {
        return Err(usage(&program, opts));
    } else if matches.free.is_empty() {
        return Err(String::from("command expected"));
    } else {
        return Ok(matches);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match parse_arguments(args) {
        Ok(m) => {
            // Arguments are valid
            let app_root = m.opt_str("a").unwrap_or(String::from("."));
            let image_name = m.opt_str("i").unwrap_or(String::from("devbase"));

            let a = RailsApp::from(app_root).unwrap();
            let i = a.image();
            println!("{}", i.dockerfile());
            i.build();
/*
            let i = a.image("devbase")
            i.build()
            i.label()

            let c = a.command("exec")
            c.run(args)
*/
        },
        Err(f) => {
            println!("{}", f.to_string());
            exit(1);
        },
    }
}
