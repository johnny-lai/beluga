//use yaml_rust::{YamlLoader, YamlEmitter};

extern crate base64;
extern crate handlebars;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use handlebars::{Handlebars, Helper, RenderContext, RenderError};

mod rsc;

//= Image ======================================================================
#[derive(Serialize, Deserialize)]
struct BuildOptions {
    from: String,
    build_instructions: String,
}

fn write_rsc (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let from = h.param(0).unwrap().value().as_str();
    let to = h.param(1).unwrap().value().as_str();
    if to == None {
        return Ok(())
    }

    rc.writer.write(b"echo ");
    if from == Some("entrypoint.sh") {
        try!(rc.writer.write(base64::encode(rsc::entrypoint_sh).into_bytes().as_ref()));
    }
    rc.writer.write(b" | base64 -d > ");
    rc.writer.write(to.unwrap().as_bytes());
    Ok(())
}

pub struct Image {
    template: String,
}

impl Image {
    fn build_instructions(&self) -> String {
        let mut str = String::new();
        str.push_str(rsc::npm_install);
        str.push_str(rsc::gem_install);
        // str.push_str(RUN apt-get install -y {{extra_packages}});
        // str.push_str(extra_build_instructions);
        return str;
    }

    fn build_options(&self) -> BuildOptions {
        return BuildOptions{
            from: String::from("ruby:2.4.3"),
            build_instructions: self.build_instructions(),
        };
    }

    fn dockerfile(&self) -> String {
        let mut reg = Handlebars::new();
        /// register the helper
        reg.register_helper("write_rsc", Box::new(write_rsc));
        return reg.render_template(&self.template, &self.build_options()).unwrap();
    }

    fn build(&self) -> io::Result<std::process::ExitStatus> {
        let mut child = Command::new("docker")
                    .arg("build")
                    .arg("-f")
                    .arg("-")
                    .arg("/Users/johnny_lai/Projects/beluga")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .spawn()
                    .expect("failed to execute process");
        {
            let mut stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(self.dockerfile().as_bytes()).expect("Failed to write to stdin");
        }

        return child.wait()
    }

    fn run(&self) {
    }
}

//= RailsApp ===================================================================
pub struct RailsApp {
    root: PathBuf
}

impl RailsApp {
    pub fn from(r: String) -> Result<RailsApp, io::Error> {
        let srcdir = PathBuf::from(r);
        match fs::canonicalize(&srcdir) {
            Ok(m) => { return Ok(RailsApp{root: m}) }
            Err(e) => return Err(e),
        };
    }

    fn image(&self) -> Image{
        return Image{template: String::from(rsc::DEVBASE)};
    }
}
