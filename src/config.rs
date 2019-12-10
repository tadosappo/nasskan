use evdev_rs::enums::EV_KEY;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum Modifier {
  Shift,
  Control,
  Alt,
  Meta,
}

impl TryFrom<EV_KEY> for Modifier {
  type Error = ();

  fn try_from(key: EV_KEY) -> Result<Self, Self::Error> {
    match key {
      EV_KEY::KEY_LEFTSHIFT | EV_KEY::KEY_RIGHTSHIFT => Ok(Self::Shift),
      EV_KEY::KEY_LEFTCTRL | EV_KEY::KEY_RIGHTCTRL => Ok(Self::Control),
      EV_KEY::KEY_LEFTALT | EV_KEY::KEY_RIGHTALT => Ok(Self::Alt),
      EV_KEY::KEY_LEFTMETA | EV_KEY::KEY_RIGHTMETA => Ok(Self::Meta),
      _ => Err(()),
    }
  }
}

impl Into<[EV_KEY; 2]> for Modifier {
  fn into(self) -> [EV_KEY; 2] {
    match self {
      Self::Shift => [EV_KEY::KEY_LEFTSHIFT, EV_KEY::KEY_RIGHTSHIFT],
      Self::Control => [EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_RIGHTCTRL],
      Self::Alt => [EV_KEY::KEY_LEFTALT, EV_KEY::KEY_RIGHTALT],
      Self::Meta => [EV_KEY::KEY_LEFTMETA, EV_KEY::KEY_RIGHTMETA],
    }
  }
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
