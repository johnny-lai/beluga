mod rsc;
pub mod rails;

use base64;
use handlebars::{Handlebars, Helper, RenderContext, RenderError, no_escape};
use pnet::datalink;
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use serde_yaml;
use sha1;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use users::{get_current_gid, get_current_uid};

fn host_public_ip() -> String {
    for iface in datalink::interfaces() {
        if iface.is_up() && !iface.is_loopback() {
            if let Some(ip) = iface.ips.iter().find(|x| x.is_ipv4()) {
                return format!("{}", ip.ip()).to_string();
            }
        }
    }
    return "".to_string();
}

fn write_rsc(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
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

//= Image ======================================================================
#[derive(Serialize, Deserialize)]
struct BuildOptions {
    from: String,
    id_rsa: String,
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

        cargs.push("-e".to_string());
        cargs.push(format!("DEV_UID={}", get_current_uid()));

        cargs.push("-e".to_string());
        cargs.push(format!("DEV_GID={}", get_current_gid()));

        cargs.push("--net=bridge".to_string());

        // Add environment
        for (k, v) in &self.def.environment {
            cargs.push("-e".to_string());
            cargs.push(format!("{}={}", k, v));
        }

        // Add hosts
        for (k, v) in &self.def.extra_hosts {
            cargs.push(format!("--add-host={}:{}", k, v));
        }

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

    #[serde(default = "CommandDef::default_extra_hosts", deserialize_with = "CommandDef::deserialize_extra_hosts")]
    pub extra_hosts: HashMap<String, String>,
}

impl CommandDef {
    fn default_image() -> String {
        "devbase".to_string()
    }

    fn default_extra_hosts() -> HashMap<String, String> {
        let mut extra_hosts = HashMap::new();
        extra_hosts.insert("db".to_string(), host_public_ip());
        return extra_hosts;
    }

    fn deserialize_extra_hosts<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
    where D: Deserializer<'de>
    {
        // This is a Visitor that forwards string types to T's `FromStr` impl and
        // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
        // keep the compiler from complaining about T being an unused generic type
        // parameter. We need T in order to know the Value type for the Visitor
        // impl.
        struct HashOrArrayVisitor;

        impl<'de> Visitor<'de> for HashOrArrayVisitor
        {
            type Value = HashMap<String, String>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("hash or array")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<String, String>, V::Error>
                where V: SeqAccess<'de>
            {
                let mut ret: HashMap<String, String> = HashMap::new();
                while let Some(s) = seq.next_element::<String>()? {
                    let v: Vec<&str> = s.split(':').collect();
                    ret.insert(v[0].to_string(), v[1].to_string());
                }
                Ok(ret)
            }

            fn visit_map<M>(self, visitor: M) -> Result<HashMap<String, String>, M::Error>
                where M: MapAccess<'de>
            {
                // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
                // into a `Deserializer`, allowing it to be used as the input to T's
                // `Deserialize` implementation. T then deserializes itself using
                // the entries from the map visitor.
                Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
            }
        }

        deserializer.deserialize_any(HashOrArrayVisitor)
    }
}

#[test]
fn deserialize_extra_hosts_as_array() {
    let cdef: CommandDef = serde_yaml::from_str(
"command: blah
extra_hosts:
  - db:123").expect("should accept extra_hosts as array");
    assert_eq!(cdef.extra_hosts.get("db").unwrap(), "123");
}

#[test]
fn deserialize_extra_hosts_as_hash() {
    let cdef: CommandDef = serde_yaml::from_str(
"command: blah
extra_hosts:
  db: 123").expect("should accept extra_hosts as hash");
    assert_eq!(cdef.extra_hosts.get("db").unwrap(), "123");
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
                extra_hosts: CommandDef::default_extra_hosts(),
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
        // Old ERB "support"
        let template = txt.replace("<%=", "{{")
                          .replace("%>", "}}");

        // Process as handlebars template
        let mut reg = Handlebars::new();
        reg.register_escape_fn(no_escape);
        let out = try!(
            reg.render_template(template.as_ref(), &json!({"host_public_ip": host_public_ip()}))
               .map_err(|e| e.to_string())
        );

        // Parse YAML to struct
        let mut ret: Config = Default::default();
        let cfg: Config = try!(serde_yaml::from_str(&out).map_err(|e| e.to_string()));
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
fn config_from_empty() {
    assert_eq!(Config::from_str(""), Err("EOF while parsing a value".to_string()));
}

// Override app.version
#[test]
fn config_from_override_app_version() {
    let mut expected: Config = Default::default();
    expected.app.version = "2".to_string();
    assert_eq!(Config::from_str(
"app:
  version: 2"
    ), Ok(expected));
}

// Override devbase.tag
#[test]
fn config_from_override_devbase_tag() {
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

// With old extra_hosts format
#[test]
fn config_from_old_extra_hosts() {
    let mut expected: Config = Default::default();
    {
        let mut extra_hosts = HashMap::new();
        extra_hosts.insert("db".to_string(), host_public_ip());

        let rspec_cmd = CommandDef{
            command: "rspec".to_string(),
            image: CommandDef::default_image(),
            environment: HashMap::new(),
            extra_hosts: extra_hosts,
        };

        expected.commands.insert("rspec".to_string(), rspec_cmd);
    }
    assert_eq!(Config::from_str(
"commands:
  rspec:
    command: rspec
    extra_hosts:
      - db:<%= host_public_ip %>"
    ), Ok(expected));
}

