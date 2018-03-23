mod rsc;
pub mod rails;

use base64;
use handlebars::{Handlebars, Helper, RenderContext, RenderError};
use serde_yaml;
use sha1;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process;

//= Image ======================================================================
#[derive(Serialize, Deserialize)]
struct BuildOptions {
    from: String,
    id_rsa: String,
}

fn write_rsc (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let from = h.param(0).unwrap().value().as_str();
    let to = h.param(1).unwrap().value().as_str();
    if to == None {
        return Ok(())
    }

    try!(rc.writer.write(b"echo "));
    let resource = match from {
        Some("entrypoint.sh") => rsc::ENTRYPOINT_SH,
        Some(&_) => "",
        None => "",
    };
    try!(rc.writer.write(base64::encode(resource).into_bytes().as_ref()));
    try!(rc.writer.write(b" | base64 -d > "));
    try!(rc.writer.write(to.unwrap().as_bytes()));
    Ok(())
}

pub struct Image {
    pub label: String,
    pub app_root: PathBuf,
    pub dockerfile: String,
}

impl Image {
    pub fn build(&self) -> io::Result<process::ExitStatus> {
        let mut child = process::Command::new("docker")
                    .arg("build")
                    .arg("-f")
                    .arg("-")
                    .arg(self.app_root.to_str().unwrap())
                    .stdin(process::Stdio::piped())
                    .stdout(process::Stdio::inherit())
                    .spawn()
                    .expect("failed to execute process");
        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(self.dockerfile.as_bytes()).expect("Failed to write to stdin");
        }

        return child.wait()
    }
}

//= Command ====================================================================
pub struct Command<'a> {
    image: Box<Image>,
    def: &'a CommandDef,
}

impl<'a> Command<'a> {
    pub fn exec(&self, args: Vec<String>) -> io::Result<process::ExitStatus>  {
        println!("{:?}", args);

        let mut cargs = Vec::<String>::new();

        cargs.push("run".to_string());
        cargs.push("--rm".to_string());
        cargs.push("-it".to_string());

        // Add volume mounts
        cargs.push("-v".to_string());
        cargs.push(format!("{}:/app", self.image.app_root.to_str().unwrap()));
        cargs.push("-w".to_string());
        cargs.push("/app".to_string());

        cargs.push("-e".to_string());
        cargs.push("IN_DOCKER=true".to_string());

        // -e DEV_UID=#{Process.uid}
        // -e DEV_GID=#{Process.gid}

        cargs.push("--net=bridge".to_string());

        // Add environment
        for (k, v) in &self.def.environment {
            cargs.push("-e".to_string());
            cargs.push(format!("{}={}", k, v));
        }

        // TODO: Add hosts

        // Image to run
        cargs.push(self.image.label.clone());

        // Command to run
        let run_cmd = self.def.command.replace("%s", &args.join(" "));
        cargs.push(run_cmd);

        println!("{:?}", cargs);

        // Spawn
        let mut child = process::Command::new("docker")
            .stdin(process::Stdio::inherit())
            .stdout(process::Stdio::inherit())
            .args(cargs)
            .spawn().expect("failed to execute process");

        return child.wait();
    }
}

//= utils ======================================================================
fn sha1_update(m: &mut sha1::Sha1, file_name: &str) {
    let contents = fs::File::open(file_name)
       .map_err(|err| err.to_string())
       .and_then(|mut file| {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|err| err.to_string())
                .map(|_| contents)
       });
    m.update(contents.unwrap().as_bytes());
}


//= Options ====================================================================
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppDef {
    #[serde(default = "String::new")]
    version: String,
}

impl AppDef {
    pub fn new() -> AppDef {
        return AppDef{version: String::new() };
    }

    pub fn extend(&mut self, rhs: AppDef) {
        if !rhs.version.is_empty() {
            self.version = rhs.version;
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageDef {
    pub tag: String,

    #[serde(default = "ImageDef::default_id_rsa")]
    pub id_rsa: String,

    #[serde(default = "String::new")]
    pub from: String,

    #[serde(default = "Vec::new")]
    pub extra_packages: Vec<String>,

    #[serde(default = "Vec::new")]
    pub extra_build_instructions: Vec<String>,
}

impl ImageDef {
    fn default_id_rsa() -> String {
        "~/.ssh/id_rsa".to_string()
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandDef {
    pub command: String,

    #[serde(default = "CommandDef::default_image")]
    pub image: String,

    #[serde(default = "HashMap::new")]
    pub environment: HashMap<String, String>,

    #[serde(default = "Vec::new")]
    pub extra_hosts: Vec<String>,
}

impl CommandDef {
    fn default_image() -> String {
        "devbase".to_string()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "AppDef::new")]
    pub app: AppDef,

    #[serde(default = "HashMap::new")]
    pub images: HashMap<String, ImageDef>,

    #[serde(default = "HashMap::new")]
    pub commands: HashMap<String, CommandDef>,
}

impl Default for Config {
    fn default() -> Config {
        let mut images = HashMap::new();
        images.insert(
            String::from("devbase"),
            ImageDef {
                tag: String::from("beluga-devbase:%s"),
                id_rsa: String::from("~/.ssh/id_rsa"),
                from: String::from("alpine"),
                extra_packages: vec![],
                extra_build_instructions: vec![],
            });

        let mut commands = HashMap::new();
        commands.insert(
            String::from("exec"),
            CommandDef {
                command: "%s".to_string(),
                image: "devbase".to_string(),
                environment: HashMap::new(),
                extra_hosts: vec![],    // TODO: Add db here?
            });

        return Config{
            app: AppDef {
                version: String::from(""),
            },
            images: images,
            commands: commands,
        };
    }
}

impl Config {
    pub fn from(p: &Path) -> Result<Config, String> {
        let contents = fs::File::open(p)
            .map_err(|err| err.to_string())
            .and_then(|mut file| {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .map_err(|err| err.to_string())
                    .map(|_| contents)
            });
        match contents {
            Ok(txt) => {
                return Config::from_str(txt.as_ref());
            }
            Err(e) => { return Err(e) }
        }
    }

    pub fn from_str(txt: &str) -> Result<Config, String> {
        let mut ret: Config = Default::default();
        let cfg: Config = try!(serde_yaml::from_str(&txt).map_err(|e| e.to_string()));
        ret.extend(cfg);
        return Ok(ret);
    }

    fn extend(&mut self, rhs: Config) {
        self.app.extend(rhs.app);
        self.images.extend(rhs.images);
        self.commands.extend(rhs.commands);
    }
}

#[test]
fn config_test_from_str() {
    assert_eq!(Config::from_str(""), Err("EOF while parsing a value".to_string()));

    { // Override app.version
        let mut expected: Config = Default::default();
        expected.app.version = "2".to_string();
        assert_eq!(Config::from_str(
"app:
  version: 2"
        ), Ok(expected));
    }

    { // Override devbase.tag
        let mut expected: Config = Default::default();
        {
            let devbase = expected.images.get_mut(&"devbase".to_string()).unwrap();
            devbase.tag = "tick-%s".to_string();
            devbase.from = "".to_string();
        }
        assert_eq!(Config::from_str(
"images:
  devbase:
    tag: tick-%s"
        ), Ok(expected));
    }
}
