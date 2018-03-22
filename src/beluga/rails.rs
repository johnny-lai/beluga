use base64;
use beluga::*;
use handlebars::{Handlebars, Helper, RenderContext, RenderError, no_escape};
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
use tilde_expand::tilde_expand;

pub struct Dockerfile<'a> {
    image_def: &'a ImageDef,
    ruby_version: &'a str,
}

impl<'a> Dockerfile<'a> {
    fn from(&self) -> String {
        format!("ruby:{}", self.ruby_version)
    }

    fn id_rsa(&self) -> String {
        let id_rsa_path = tilde_expand(self.image_def.id_rsa.as_bytes());

        let mut f = fs::File::open(String::from_utf8(id_rsa_path).unwrap()).expect("id_rsa required");

        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("failed to read id_rsa");

        return base64::encode(&contents);
    }

    fn build_options(&self) -> BuildOptions {
        return BuildOptions {
            from: self.from(),
            id_rsa: self.id_rsa(),
        };
    }

    fn template(&self) -> String {
        let mut str = rsc::DEVBASE.to_string();
        
        str.push_str(rsc::NPM_INSTALL);
        str.push_str(rsc::GEM_INSTALL);

        let pkgs = &self.image_def.extra_packages;
        if !pkgs.is_empty() {
            str.push_str("RUN apt-get install -y ");
            for i in pkgs.iter() {
                str.push_str(i);
                str.push_str(" ");
            }
            str.push_str("\n");
        }

        for i in self.image_def.extra_build_instructions.iter() {
            str.push_str(i);
            str.push_str("\n");
        }

        return str;
    }

    pub fn to_string(&self) -> String {
        let mut reg = Handlebars::new();
        reg.register_escape_fn(no_escape);
        reg.register_helper("write_rsc", Box::new(write_rsc));
        return reg.render_template(&self.template(), &self.build_options()).unwrap();
    }
}

//= RailsApp ===================================================================
pub struct App {
    pub root: PathBuf,
    pub config: Config,

    ruby_version: String,
}

impl App {
    pub fn from(r: String) -> Result<App, String> {
        let srcdir = PathBuf::from(r);
        fs::canonicalize(&srcdir)
            .map_err(|e| e.to_string())
            .map(|m| {
                let mut cfg_path = m.clone();
                cfg_path.push("config");
                cfg_path.push("beluga.yml");

                let mut config = Config::from(cfg_path.as_path()).unwrap();
                let ruby_version = {
                    // Read .ruby-version
                    let mut f = fs::File::open(".ruby-version").expect(".ruby-version required");

                    let mut contents = String::new();
                    f.read_to_string(&mut contents).expect("failed to read .ruby-version");

                    contents
                };
                App {
                    root: m,
                    config: config,
                    ruby_version: ruby_version,
                }
            })
    }

    fn image_label(&self, image_tag: &str) -> String {
        return image_tag.replace("%s", &self.digest().unwrap());
    }

    pub fn digest(&self) -> Result<String, String> {
        let mut m = sha1::Sha1::new();

        let version = &self.config.app.version;
        m.update(version.as_bytes());

        // .ruby-version package.json npm-shrinkwrap.json Gemfile Gemfile.lock
        //m.update(self.ruby_version.as_bytes());
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
                let label = { self.image_label(imgdef.tag.as_ref()) };
                let d = Dockerfile{
                    image_def: imgdef,
                    ruby_version: &self.ruby_version,
                };
                return Some(Image{
                    label: label,
                    app_root: self.root.clone(),
                    dockerfile: d.to_string(),
                });
            },
            None => { return None }
        }
    }

    pub fn command(&self, name: &str) -> Option<Command> {
         match self.config.commands.get(name) {
            Some(cdef) => {
                let image = self.image(&cdef.image).unwrap();
                return Some(Command{
                    image: Box::new(image),
                    def: cdef,
                });
            },
            None => { return None }
        }
    }
}
