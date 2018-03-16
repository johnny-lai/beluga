mod rsc;

use base64;
use handlebars::{Handlebars, Helper, RenderContext, RenderError};
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio, ExitStatus};
//use yaml_rust::{YamlLoader, YamlEmitter};

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
        try!(rc.writer.write(base64::encode(rsc::ENTRYPOINT_SH).into_bytes().as_ref()));
    }
    rc.writer.write(b" | base64 -d > ");
    rc.writer.write(to.unwrap().as_bytes());
    Ok(())
}

pub struct Image<'a> {
    label: String,
    app_root: &'a PathBuf,
    template: String,
}

impl<'a> Image<'a> {
    fn build_instructions(&self) -> String {
        let mut str = String::new();
        str.push_str(rsc::NPM_INSTALL);
        str.push_str(rsc::GEM_INSTALL);
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

    pub fn dockerfile(&self) -> String {
        let mut reg = Handlebars::new();
        reg.register_helper("write_rsc", Box::new(write_rsc));
        return reg.render_template(&self.template, &self.build_options()).unwrap();
    }

    pub fn build(&self) -> io::Result<ExitStatus> {
        let mut child = Command::new("docker")
                    .arg("build")
                    .arg("-f")
                    .arg("-")
                    .arg(self.app_root.to_str().unwrap())
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .spawn()
                    .expect("failed to execute process");
        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(self.dockerfile().as_bytes()).expect("Failed to write to stdin");
        }

        return child.wait()
    }

    pub fn exec(&self) {
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

    fn image_label(&self, image_name: &str) -> String {
        return format!("{}:{}", image_name, self.digest());
    }

    pub fn digest(&self) -> String {
        return String::from("digest");
    }

    pub fn image(&self, image_name: &str) -> Image {
        return Image{
            label: self.image_label(image_name),
            app_root: &self.root,
            template: String::from(rsc::DEVBASE)
        };
    }
}
