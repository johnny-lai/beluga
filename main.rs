extern crate base64;
extern crate getopts;
extern crate handlebars;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate yaml_rust;

mod beluga;

use beluga::RailsApp;
use getopts::Options;
use getopts::Matches;
use std::env;
use std::process::exit;

fn usage(program: &str, opts: &Options) -> String{
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

fn parse_arguments(opts: &Options, args: &Vec<String>) -> Result<Matches, String> {
    let program = args[0].clone();


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

fn run(opts: &Options, args: &Vec<String>) -> Result<(), String> {
    let m = try!(parse_arguments(&opts, &args));

    // Arguments are valid
    let app_root = m.opt_str("a").unwrap_or(String::from("."));
    let image_name = m.opt_str("i").unwrap_or(String::from("devbase"));

    // TODO: Handle bad app_root
    let app = RailsApp::from(app_root).unwrap();

    // TODO: Handle bad image
    let image = app.image(&image_name);

    let mut i = 0;
    let args = m.free;
    if i >= args.len() {
        return Err(String::from("argument missing"));
    }
    match args[i].as_ref() {
        "digest" => {
            println!("{}", app.digest())
        },
        "command" => {
        },
        "image" => {
            i += 1;
            if i >= args.len() {
                return Err(String::from("argument missing"));
            }
            match args[i].as_ref() {
                "list" => {
                },
                "info" => {
                },
                "label" => {
                },
                "build" => {
                    println!("building");
                    image.build();
                },
                "push" => {
                },
                "pull" => {
                },
                "clean" => {
                },
                _ => return Err(String::from("unknown image command")),
            }
        },
        _ => return Err(String::from("unknown command")),
    }

    return Ok(())
}

fn main() {
    let mut opts = Options::new();
    opts.optopt("a", "", "Location of Application. Defaults to '.'", "APP");
    opts.optopt("i", "", "Name of image. Defaults to devbase", "IMAGE");
    opts.optflag("h", "help", "print this help menu");

    let args: Vec<String> = env::args().collect();

    match run(&opts, &args) {
      Ok(_) => {},
      Err(msg) => {
        println!("{}", msg.to_string());
        exit(1);
      }
    }
}
