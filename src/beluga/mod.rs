mod rsc;

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
use std::process::{Command, Stdio, ExitStatus};

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
    dockerfile: String,
}

impl<'a> Image<'a> {
    pub fn build(&self) -> io::Result<ExitStatus> {
        println!("{}", self.dockerfile);
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
            stdin.write_all(self.dockerfile.as_bytes()).expect("Failed to write to stdin");
        }

        return child.wait()
    }

    pub fn exec(&self) {
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
    tag: String,

    #[serde(default = "ImageDef::default_id_rsa")]
    id_rsa: String,

    #[serde(default = "String::new")]
    from: String,

    #[serde(default = "Vec::new")]
    extra_packages: Vec<String>,

    #[serde(default = "Vec::new")]
    extra_build_instructions: Vec<String>,
}

impl ImageDef {
    fn default_id_rsa() -> String {
        "~/.ssh/id_rsa".to_string()
    }

    fn build_instructions(&self) -> String {
        let mut str = String::new();
        str.push_str(rsc::NPM_INSTALL);
        str.push_str(rsc::GEM_INSTALL);
        // str.push_str(RUN apt-get install -y {{extra_packages}});
        // str.push_str(extra_build_instructions);
        return str;
    }

    fn build_options(&self) -> BuildOptions {
        return BuildOptions {
            from: self.from.clone(),
            build_instructions: self.build_instructions(),
        };
    }

    fn dockerfile_from(&self, template: String) -> String {
        let mut reg = Handlebars::new();
        reg.register_helper("write_rsc", Box::new(write_rsc));
        return reg.render_template(&template, &self.build_options()).unwrap();
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandDef {
    command: String,

    #[serde(default = "CommandDef::default_image")]
    image: String,

    #[serde(default = "HashMap::new")]
    environment: HashMap<String, String>,

    #[serde(default = "Vec::new")]
    extra_hosts: Vec<String>,
}

impl CommandDef {
    fn default_image() -> String {
        "devbase".to_string()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "AppDef::new")]
    app: AppDef,

    #[serde(default = "HashMap::new")]
    images: HashMap<String, ImageDef>,

    #[serde(default = "HashMap::new")]
    commands: HashMap<String, CommandDef>,
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
        return Config{
            app: AppDef {
                version: String::from(""),
            },
            images: images ,
            commands: HashMap::new(),
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
        println!("{:?}", ret);
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

//= RailsApp ===================================================================
pub struct RailsApp {
    root: PathBuf,
    config: Config,
}

impl RailsApp {
    pub fn from(r: String) -> Result<RailsApp, String> {
        let srcdir = PathBuf::from(r);
        fs::canonicalize(&srcdir)
            .map_err(|e| e.to_string())
            .map(|m| {
                let mut cfg_path = m.clone();
                cfg_path.push("config");
                cfg_path.push("beluga.yml");

                let mut config = Config::from(cfg_path.as_path()).unwrap();
                {
                    let devbase = config.images.get_mut("devbase").unwrap();
                    if devbase.from.is_empty() {
                        // TODO: Read .ruby-version here
                        devbase.from = "ruby:2.4.3".to_string();
                    }
                }
                RailsApp {
                    root: m,
                    config: config,
                }
            })
    }

    fn image_label(&self, image_name: &str) -> String {
        // TODO: Perform sprintf
        return format!("{}:{}", image_name, self.digest().unwrap());
    }

    pub fn digest(&self) -> Result<String, String> {
        let mut m = sha1::Sha1::new();

        let version = &self.config.app.version;
        m.update(version.as_bytes());

        // .ruby-version package.json npm-shrinkwrap.json Gemfile Gemfile.lock
        sha1_update(&mut m, ".ruby-version");
        sha1_update(&mut m, "package.json");
        sha1_update(&mut m, "npm-shrinkwrap.json");
        sha1_update(&mut m, "Gemfile");
        sha1_update(&mut m, "Gemfile.lock");

        return Ok(m.digest().to_string());
    }

    pub fn image(&self, name: &str) -> Option<Image> {
        match self.config.images.get(name) {
            Some(imgdef) => {
                return Some(Image{
                    label: self.image_label(imgdef.tag.as_ref()),
                    app_root: &self.root,
                    dockerfile: imgdef.dockerfile_from(String::from(rsc::DEVBASE)),
                });
            },
            None => { return None }
        }
    }
}
