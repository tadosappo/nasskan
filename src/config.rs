use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub version: u8,
  pub devices: Vec<Device>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
  #[serde(rename(deserialize = "if"))]
  pub if_: HashMap<String, String>,
  pub then: Vec<Rule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
  pub from: Flom,
  pub to: To,
  pub tap: Option<Tap>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Flom {
  pub key: String,
  pub with: Option<Vec<Modifier>>,
  pub without: Option<Vec<Modifier>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct To {
  pub key: String,
  pub with: Option<Vec<Modifier>>,
  pub without: Option<Vec<Modifier>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tap {
  pub key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Modifier {
  Shift,
  Control,
  Alt,
  Meta,
}

lazy_static! {
  pub static ref CONFIG: Config = {
    let file = std::fs::File::open("/etc/nasskan/config.yaml")
      .expect("/etc/nasskan/config.yaml could not be opened");
    let reader = std::io::BufReader::new(file);
    let config: Config =
      serde_yaml::from_reader(reader).expect("/etc/nasskan/config.yaml has invalid shape");

    assert_eq!(config.version, 1);
    config
  };
}
