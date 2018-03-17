use std::fs;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::Read;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppDef {
    pub version: Option<String>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageDef {
    pub tag: Option<String>,
    pub id_rsa: Option<String>,
    pub from: Option<String>,
    pub extra_packages: Option<Vec<String>>,
    pub extra_build_instructions: Option<Vec<String>>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandDef {
    pub command: String,
    pub image: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub extra_hosts: Option<Vec<String>>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub app: Option<AppDef>,
    pub images: HashMap<String, ImageDef>,
    pub commands: HashMap<String, CommandDef>,
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
                let ret: Config = serde_yaml::from_str(&txt).unwrap();
                println!("{:?}", ret);
                return Ok(ret);
            }
            Err(e) => { return Err(e) }
        }
    }
}
