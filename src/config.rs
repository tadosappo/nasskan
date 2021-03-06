use evdev_rs::enums::EV_KEY;
use lazy_static::lazy_static;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::cmp::{Ordering, PartialOrd};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

mod validation;
use validation::*;

lazy_static! {
  pub(crate) static ref CONFIG: Config = {
    let file = std::fs::File::open("/etc/nasskan/config.yaml")
      .expect("/etc/nasskan/config.yaml could not be opened");
    let reader = std::io::BufReader::new(file);
    let config: Config =
      serde_yaml::from_reader(reader).expect("/etc/nasskan/config.yaml has invalid shape");

    validate_order(&config);
    validate_tap(&config);

    assert_eq!(config.version, 1);
    config
  };
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Config {
  pub(crate) version: u8,
  pub(crate) devices: Vec<Device>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Device {
  #[serde(rename(deserialize = "if"))]
  pub(crate) if_: BTreeMap<String, String>,
  pub(crate) then: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Rule {
  pub(crate) from: From_,
  pub(crate) to: To,
  pub(crate) tap: Option<Tap>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct From_ {
  pub(crate) key: EventKey,
  pub(crate) with: Option<BTreeSet<Modifier>>,
  pub(crate) without: Option<BTreeSet<Modifier>>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct To {
  pub(crate) key: EventKey,
  pub(crate) with: Option<BTreeSet<Modifier>>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Tap {
  pub(crate) key: EventKey,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Modifier {
  LEFTSHIFT,
  RIGHTSHIFT,
  LEFTCTRL,
  RIGHTCTRL,
  LEFTALT,
  RIGHTALT,
  LEFTMETA,
  RIGHTMETA,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EventKey(EV_KEY);

impl From<EV_KEY> for EventKey {
  fn from(key: EV_KEY) -> Self {
    Self(key)
  }
}

impl Into<EV_KEY> for EventKey {
  fn into(self) -> EV_KEY {
    self.0
  }
}

impl Deref for EventKey {
  type Target = EV_KEY;

  fn deref(&self) -> &EV_KEY {
    &self.0
  }
}

impl Ord for EventKey {
  fn cmp(&self, other: &Self) -> Ordering {
    (self.0.clone() as u32).cmp(&(other.0.clone() as u32))
  }
}

impl PartialOrd for EventKey {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    (self.0.clone() as u32).partial_cmp(&(other.0.clone() as u32))
  }
}

impl<'a> Deserialize<'a> for EventKey {
  fn deserialize<T: Deserializer<'a>>(deserializer: T) -> Result<Self, T::Error> {
    deserializer.deserialize_str(EventKeyVisitor)
  }
}

struct EventKeyVisitor;
impl<'a> Visitor<'a> for EventKeyVisitor {
  type Value = EventKey;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    formatter.write_str("keycode name")
  }

  fn visit_str<T: serde::de::Error>(self, value: &str) -> Result<Self::Value, T> {
    match value {
      "RESERVED" => Ok(EventKey(EV_KEY::KEY_RESERVED)),
      "ESC" => Ok(EventKey(EV_KEY::KEY_ESC)),
      "1" => Ok(EventKey(EV_KEY::KEY_1)),
      "2" => Ok(EventKey(EV_KEY::KEY_2)),
      "3" => Ok(EventKey(EV_KEY::KEY_3)),
      "4" => Ok(EventKey(EV_KEY::KEY_4)),
      "5" => Ok(EventKey(EV_KEY::KEY_5)),
      "6" => Ok(EventKey(EV_KEY::KEY_6)),
      "7" => Ok(EventKey(EV_KEY::KEY_7)),
      "8" => Ok(EventKey(EV_KEY::KEY_8)),
      "9" => Ok(EventKey(EV_KEY::KEY_9)),
      "0" => Ok(EventKey(EV_KEY::KEY_0)),
      "MINUS" => Ok(EventKey(EV_KEY::KEY_MINUS)),
      "EQUAL" => Ok(EventKey(EV_KEY::KEY_EQUAL)),
      "BACKSPACE" => Ok(EventKey(EV_KEY::KEY_BACKSPACE)),
      "TAB" => Ok(EventKey(EV_KEY::KEY_TAB)),
      "Q" => Ok(EventKey(EV_KEY::KEY_Q)),
      "W" => Ok(EventKey(EV_KEY::KEY_W)),
      "E" => Ok(EventKey(EV_KEY::KEY_E)),
      "R" => Ok(EventKey(EV_KEY::KEY_R)),
      "T" => Ok(EventKey(EV_KEY::KEY_T)),
      "Y" => Ok(EventKey(EV_KEY::KEY_Y)),
      "U" => Ok(EventKey(EV_KEY::KEY_U)),
      "I" => Ok(EventKey(EV_KEY::KEY_I)),
      "O" => Ok(EventKey(EV_KEY::KEY_O)),
      "P" => Ok(EventKey(EV_KEY::KEY_P)),
      "LEFTBRACE" => Ok(EventKey(EV_KEY::KEY_LEFTBRACE)),
      "RIGHTBRACE" => Ok(EventKey(EV_KEY::KEY_RIGHTBRACE)),
      "ENTER" => Ok(EventKey(EV_KEY::KEY_ENTER)),
      "LEFTCTRL" => Ok(EventKey(EV_KEY::KEY_LEFTCTRL)),
      "A" => Ok(EventKey(EV_KEY::KEY_A)),
      "S" => Ok(EventKey(EV_KEY::KEY_S)),
      "D" => Ok(EventKey(EV_KEY::KEY_D)),
      "F" => Ok(EventKey(EV_KEY::KEY_F)),
      "G" => Ok(EventKey(EV_KEY::KEY_G)),
      "H" => Ok(EventKey(EV_KEY::KEY_H)),
      "J" => Ok(EventKey(EV_KEY::KEY_J)),
      "K" => Ok(EventKey(EV_KEY::KEY_K)),
      "L" => Ok(EventKey(EV_KEY::KEY_L)),
      "SEMICOLON" => Ok(EventKey(EV_KEY::KEY_SEMICOLON)),
      "APOSTROPHE" => Ok(EventKey(EV_KEY::KEY_APOSTROPHE)),
      "GRAVE" => Ok(EventKey(EV_KEY::KEY_GRAVE)),
      "LEFTSHIFT" => Ok(EventKey(EV_KEY::KEY_LEFTSHIFT)),
      "BACKSLASH" => Ok(EventKey(EV_KEY::KEY_BACKSLASH)),
      "Z" => Ok(EventKey(EV_KEY::KEY_Z)),
      "X" => Ok(EventKey(EV_KEY::KEY_X)),
      "C" => Ok(EventKey(EV_KEY::KEY_C)),
      "V" => Ok(EventKey(EV_KEY::KEY_V)),
      "B" => Ok(EventKey(EV_KEY::KEY_B)),
      "N" => Ok(EventKey(EV_KEY::KEY_N)),
      "M" => Ok(EventKey(EV_KEY::KEY_M)),
      "COMMA" => Ok(EventKey(EV_KEY::KEY_COMMA)),
      "DOT" => Ok(EventKey(EV_KEY::KEY_DOT)),
      "SLASH" => Ok(EventKey(EV_KEY::KEY_SLASH)),
      "RIGHTSHIFT" => Ok(EventKey(EV_KEY::KEY_RIGHTSHIFT)),
      "KPASTERISK" => Ok(EventKey(EV_KEY::KEY_KPASTERISK)),
      "LEFTALT" => Ok(EventKey(EV_KEY::KEY_LEFTALT)),
      "SPACE" => Ok(EventKey(EV_KEY::KEY_SPACE)),
      "CAPSLOCK" => Ok(EventKey(EV_KEY::KEY_CAPSLOCK)),
      "F1" => Ok(EventKey(EV_KEY::KEY_F1)),
      "F2" => Ok(EventKey(EV_KEY::KEY_F2)),
      "F3" => Ok(EventKey(EV_KEY::KEY_F3)),
      "F4" => Ok(EventKey(EV_KEY::KEY_F4)),
      "F5" => Ok(EventKey(EV_KEY::KEY_F5)),
      "F6" => Ok(EventKey(EV_KEY::KEY_F6)),
      "F7" => Ok(EventKey(EV_KEY::KEY_F7)),
      "F8" => Ok(EventKey(EV_KEY::KEY_F8)),
      "F9" => Ok(EventKey(EV_KEY::KEY_F9)),
      "F10" => Ok(EventKey(EV_KEY::KEY_F10)),
      "NUMLOCK" => Ok(EventKey(EV_KEY::KEY_NUMLOCK)),
      "SCROLLLOCK" => Ok(EventKey(EV_KEY::KEY_SCROLLLOCK)),
      "KP7" => Ok(EventKey(EV_KEY::KEY_KP7)),
      "KP8" => Ok(EventKey(EV_KEY::KEY_KP8)),
      "KP9" => Ok(EventKey(EV_KEY::KEY_KP9)),
      "KPMINUS" => Ok(EventKey(EV_KEY::KEY_KPMINUS)),
      "KP4" => Ok(EventKey(EV_KEY::KEY_KP4)),
      "KP5" => Ok(EventKey(EV_KEY::KEY_KP5)),
      "KP6" => Ok(EventKey(EV_KEY::KEY_KP6)),
      "KPPLUS" => Ok(EventKey(EV_KEY::KEY_KPPLUS)),
      "KP1" => Ok(EventKey(EV_KEY::KEY_KP1)),
      "KP2" => Ok(EventKey(EV_KEY::KEY_KP2)),
      "KP3" => Ok(EventKey(EV_KEY::KEY_KP3)),
      "KP0" => Ok(EventKey(EV_KEY::KEY_KP0)),
      "KPDOT" => Ok(EventKey(EV_KEY::KEY_KPDOT)),
      "ZENKAKUHANKAKU" => Ok(EventKey(EV_KEY::KEY_ZENKAKUHANKAKU)),
      "102ND" => Ok(EventKey(EV_KEY::KEY_102ND)),
      "F11" => Ok(EventKey(EV_KEY::KEY_F11)),
      "F12" => Ok(EventKey(EV_KEY::KEY_F12)),
      "RO" => Ok(EventKey(EV_KEY::KEY_RO)),
      "KATAKANA" => Ok(EventKey(EV_KEY::KEY_KATAKANA)),
      "HIRAGANA" => Ok(EventKey(EV_KEY::KEY_HIRAGANA)),
      "HENKAN" => Ok(EventKey(EV_KEY::KEY_HENKAN)),
      "KATAKANAHIRAGANA" => Ok(EventKey(EV_KEY::KEY_KATAKANAHIRAGANA)),
      "MUHENKAN" => Ok(EventKey(EV_KEY::KEY_MUHENKAN)),
      "KPJPCOMMA" => Ok(EventKey(EV_KEY::KEY_KPJPCOMMA)),
      "KPENTER" => Ok(EventKey(EV_KEY::KEY_KPENTER)),
      "RIGHTCTRL" => Ok(EventKey(EV_KEY::KEY_RIGHTCTRL)),
      "KPSLASH" => Ok(EventKey(EV_KEY::KEY_KPSLASH)),
      "SYSRQ" => Ok(EventKey(EV_KEY::KEY_SYSRQ)),
      "RIGHTALT" => Ok(EventKey(EV_KEY::KEY_RIGHTALT)),
      "LINEFEED" => Ok(EventKey(EV_KEY::KEY_LINEFEED)),
      "HOME" => Ok(EventKey(EV_KEY::KEY_HOME)),
      "UP" => Ok(EventKey(EV_KEY::KEY_UP)),
      "PAGEUP" => Ok(EventKey(EV_KEY::KEY_PAGEUP)),
      "LEFT" => Ok(EventKey(EV_KEY::KEY_LEFT)),
      "RIGHT" => Ok(EventKey(EV_KEY::KEY_RIGHT)),
      "END" => Ok(EventKey(EV_KEY::KEY_END)),
      "DOWN" => Ok(EventKey(EV_KEY::KEY_DOWN)),
      "PAGEDOWN" => Ok(EventKey(EV_KEY::KEY_PAGEDOWN)),
      "INSERT" => Ok(EventKey(EV_KEY::KEY_INSERT)),
      "DELETE" => Ok(EventKey(EV_KEY::KEY_DELETE)),
      "MACRO" => Ok(EventKey(EV_KEY::KEY_MACRO)),
      "MUTE" => Ok(EventKey(EV_KEY::KEY_MUTE)),
      "VOLUMEDOWN" => Ok(EventKey(EV_KEY::KEY_VOLUMEDOWN)),
      "VOLUMEUP" => Ok(EventKey(EV_KEY::KEY_VOLUMEUP)),
      "POWER" => Ok(EventKey(EV_KEY::KEY_POWER)),
      "KPEQUAL" => Ok(EventKey(EV_KEY::KEY_KPEQUAL)),
      "KPPLUSMINUS" => Ok(EventKey(EV_KEY::KEY_KPPLUSMINUS)),
      "PAUSE" => Ok(EventKey(EV_KEY::KEY_PAUSE)),
      "SCALE" => Ok(EventKey(EV_KEY::KEY_SCALE)),
      "KPCOMMA" => Ok(EventKey(EV_KEY::KEY_KPCOMMA)),
      "HANGEUL" => Ok(EventKey(EV_KEY::KEY_HANGEUL)),
      "HANJA" => Ok(EventKey(EV_KEY::KEY_HANJA)),
      "YEN" => Ok(EventKey(EV_KEY::KEY_YEN)),
      "LEFTMETA" => Ok(EventKey(EV_KEY::KEY_LEFTMETA)),
      "RIGHTMETA" => Ok(EventKey(EV_KEY::KEY_RIGHTMETA)),
      "COMPOSE" => Ok(EventKey(EV_KEY::KEY_COMPOSE)),
      "STOP" => Ok(EventKey(EV_KEY::KEY_STOP)),
      "AGAIN" => Ok(EventKey(EV_KEY::KEY_AGAIN)),
      "PROPS" => Ok(EventKey(EV_KEY::KEY_PROPS)),
      "UNDO" => Ok(EventKey(EV_KEY::KEY_UNDO)),
      "FRONT" => Ok(EventKey(EV_KEY::KEY_FRONT)),
      "COPY" => Ok(EventKey(EV_KEY::KEY_COPY)),
      "OPEN" => Ok(EventKey(EV_KEY::KEY_OPEN)),
      "PASTE" => Ok(EventKey(EV_KEY::KEY_PASTE)),
      "FIND" => Ok(EventKey(EV_KEY::KEY_FIND)),
      "CUT" => Ok(EventKey(EV_KEY::KEY_CUT)),
      "HELP" => Ok(EventKey(EV_KEY::KEY_HELP)),
      "MENU" => Ok(EventKey(EV_KEY::KEY_MENU)),
      "CALC" => Ok(EventKey(EV_KEY::KEY_CALC)),
      "SETUP" => Ok(EventKey(EV_KEY::KEY_SETUP)),
      "SLEEP" => Ok(EventKey(EV_KEY::KEY_SLEEP)),
      "WAKEUP" => Ok(EventKey(EV_KEY::KEY_WAKEUP)),
      "FILE" => Ok(EventKey(EV_KEY::KEY_FILE)),
      "SENDFILE" => Ok(EventKey(EV_KEY::KEY_SENDFILE)),
      "DELETEFILE" => Ok(EventKey(EV_KEY::KEY_DELETEFILE)),
      "XFER" => Ok(EventKey(EV_KEY::KEY_XFER)),
      "PROG1" => Ok(EventKey(EV_KEY::KEY_PROG1)),
      "PROG2" => Ok(EventKey(EV_KEY::KEY_PROG2)),
      "WWW" => Ok(EventKey(EV_KEY::KEY_WWW)),
      "MSDOS" => Ok(EventKey(EV_KEY::KEY_MSDOS)),
      "COFFEE" => Ok(EventKey(EV_KEY::KEY_COFFEE)),
      "ROTATE_DISPLAY" => Ok(EventKey(EV_KEY::KEY_ROTATE_DISPLAY)),
      "CYCLEWINDOWS" => Ok(EventKey(EV_KEY::KEY_CYCLEWINDOWS)),
      "MAIL" => Ok(EventKey(EV_KEY::KEY_MAIL)),
      "BOOKMARKS" => Ok(EventKey(EV_KEY::KEY_BOOKMARKS)),
      "COMPUTER" => Ok(EventKey(EV_KEY::KEY_COMPUTER)),
      "BACK" => Ok(EventKey(EV_KEY::KEY_BACK)),
      "FORWARD" => Ok(EventKey(EV_KEY::KEY_FORWARD)),
      "CLOSECD" => Ok(EventKey(EV_KEY::KEY_CLOSECD)),
      "EJECTCD" => Ok(EventKey(EV_KEY::KEY_EJECTCD)),
      "EJECTCLOSECD" => Ok(EventKey(EV_KEY::KEY_EJECTCLOSECD)),
      "NEXTSONG" => Ok(EventKey(EV_KEY::KEY_NEXTSONG)),
      "PLAYPAUSE" => Ok(EventKey(EV_KEY::KEY_PLAYPAUSE)),
      "PREVIOUSSONG" => Ok(EventKey(EV_KEY::KEY_PREVIOUSSONG)),
      "STOPCD" => Ok(EventKey(EV_KEY::KEY_STOPCD)),
      "RECORD" => Ok(EventKey(EV_KEY::KEY_RECORD)),
      "REWIND" => Ok(EventKey(EV_KEY::KEY_REWIND)),
      "PHONE" => Ok(EventKey(EV_KEY::KEY_PHONE)),
      "ISO" => Ok(EventKey(EV_KEY::KEY_ISO)),
      "CONFIG" => Ok(EventKey(EV_KEY::KEY_CONFIG)),
      "HOMEPAGE" => Ok(EventKey(EV_KEY::KEY_HOMEPAGE)),
      "REFRESH" => Ok(EventKey(EV_KEY::KEY_REFRESH)),
      "EXIT" => Ok(EventKey(EV_KEY::KEY_EXIT)),
      "MOVE" => Ok(EventKey(EV_KEY::KEY_MOVE)),
      "EDIT" => Ok(EventKey(EV_KEY::KEY_EDIT)),
      "SCROLLUP" => Ok(EventKey(EV_KEY::KEY_SCROLLUP)),
      "SCROLLDOWN" => Ok(EventKey(EV_KEY::KEY_SCROLLDOWN)),
      "KPLEFTPAREN" => Ok(EventKey(EV_KEY::KEY_KPLEFTPAREN)),
      "KPRIGHTPAREN" => Ok(EventKey(EV_KEY::KEY_KPRIGHTPAREN)),
      "NEW" => Ok(EventKey(EV_KEY::KEY_NEW)),
      "REDO" => Ok(EventKey(EV_KEY::KEY_REDO)),
      "F13" => Ok(EventKey(EV_KEY::KEY_F13)),
      "F14" => Ok(EventKey(EV_KEY::KEY_F14)),
      "F15" => Ok(EventKey(EV_KEY::KEY_F15)),
      "F16" => Ok(EventKey(EV_KEY::KEY_F16)),
      "F17" => Ok(EventKey(EV_KEY::KEY_F17)),
      "F18" => Ok(EventKey(EV_KEY::KEY_F18)),
      "F19" => Ok(EventKey(EV_KEY::KEY_F19)),
      "F20" => Ok(EventKey(EV_KEY::KEY_F20)),
      "F21" => Ok(EventKey(EV_KEY::KEY_F21)),
      "F22" => Ok(EventKey(EV_KEY::KEY_F22)),
      "F23" => Ok(EventKey(EV_KEY::KEY_F23)),
      "F24" => Ok(EventKey(EV_KEY::KEY_F24)),
      "PLAYCD" => Ok(EventKey(EV_KEY::KEY_PLAYCD)),
      "PAUSECD" => Ok(EventKey(EV_KEY::KEY_PAUSECD)),
      "PROG3" => Ok(EventKey(EV_KEY::KEY_PROG3)),
      "PROG4" => Ok(EventKey(EV_KEY::KEY_PROG4)),
      "DASHBOARD" => Ok(EventKey(EV_KEY::KEY_DASHBOARD)),
      "SUSPEND" => Ok(EventKey(EV_KEY::KEY_SUSPEND)),
      "CLOSE" => Ok(EventKey(EV_KEY::KEY_CLOSE)),
      "PLAY" => Ok(EventKey(EV_KEY::KEY_PLAY)),
      "FASTFORWARD" => Ok(EventKey(EV_KEY::KEY_FASTFORWARD)),
      "BASSBOOST" => Ok(EventKey(EV_KEY::KEY_BASSBOOST)),
      "PRINT" => Ok(EventKey(EV_KEY::KEY_PRINT)),
      "HP" => Ok(EventKey(EV_KEY::KEY_HP)),
      "CAMERA" => Ok(EventKey(EV_KEY::KEY_CAMERA)),
      "SOUND" => Ok(EventKey(EV_KEY::KEY_SOUND)),
      "QUESTION" => Ok(EventKey(EV_KEY::KEY_QUESTION)),
      "EMAIL" => Ok(EventKey(EV_KEY::KEY_EMAIL)),
      "CHAT" => Ok(EventKey(EV_KEY::KEY_CHAT)),
      "SEARCH" => Ok(EventKey(EV_KEY::KEY_SEARCH)),
      "CONNECT" => Ok(EventKey(EV_KEY::KEY_CONNECT)),
      "FINANCE" => Ok(EventKey(EV_KEY::KEY_FINANCE)),
      "SPORT" => Ok(EventKey(EV_KEY::KEY_SPORT)),
      "SHOP" => Ok(EventKey(EV_KEY::KEY_SHOP)),
      "ALTERASE" => Ok(EventKey(EV_KEY::KEY_ALTERASE)),
      "CANCEL" => Ok(EventKey(EV_KEY::KEY_CANCEL)),
      "BRIGHTNESSDOWN" => Ok(EventKey(EV_KEY::KEY_BRIGHTNESSDOWN)),
      "BRIGHTNESSUP" => Ok(EventKey(EV_KEY::KEY_BRIGHTNESSUP)),
      "MEDIA" => Ok(EventKey(EV_KEY::KEY_MEDIA)),
      "SWITCHVIDEOMODE" => Ok(EventKey(EV_KEY::KEY_SWITCHVIDEOMODE)),
      "KBDILLUMTOGGLE" => Ok(EventKey(EV_KEY::KEY_KBDILLUMTOGGLE)),
      "KBDILLUMDOWN" => Ok(EventKey(EV_KEY::KEY_KBDILLUMDOWN)),
      "KBDILLUMUP" => Ok(EventKey(EV_KEY::KEY_KBDILLUMUP)),
      "SEND" => Ok(EventKey(EV_KEY::KEY_SEND)),
      "REPLY" => Ok(EventKey(EV_KEY::KEY_REPLY)),
      "FORWARDMAIL" => Ok(EventKey(EV_KEY::KEY_FORWARDMAIL)),
      "SAVE" => Ok(EventKey(EV_KEY::KEY_SAVE)),
      "DOCUMENTS" => Ok(EventKey(EV_KEY::KEY_DOCUMENTS)),
      "BATTERY" => Ok(EventKey(EV_KEY::KEY_BATTERY)),
      "BLUETOOTH" => Ok(EventKey(EV_KEY::KEY_BLUETOOTH)),
      "WLAN" => Ok(EventKey(EV_KEY::KEY_WLAN)),
      "UWB" => Ok(EventKey(EV_KEY::KEY_UWB)),
      "UNKNOWN" => Ok(EventKey(EV_KEY::KEY_UNKNOWN)),
      "VIDEO_NEXT" => Ok(EventKey(EV_KEY::KEY_VIDEO_NEXT)),
      "VIDEO_PREV" => Ok(EventKey(EV_KEY::KEY_VIDEO_PREV)),
      "BRIGHTNESS_CYCLE" => Ok(EventKey(EV_KEY::KEY_BRIGHTNESS_CYCLE)),
      "BRIGHTNESS_AUTO" => Ok(EventKey(EV_KEY::KEY_BRIGHTNESS_AUTO)),
      "DISPLAY_OFF" => Ok(EventKey(EV_KEY::KEY_DISPLAY_OFF)),
      "WWAN" => Ok(EventKey(EV_KEY::KEY_WWAN)),
      "RFKILL" => Ok(EventKey(EV_KEY::KEY_RFKILL)),
      "MICMUTE" => Ok(EventKey(EV_KEY::KEY_MICMUTE)),
      "OK" => Ok(EventKey(EV_KEY::KEY_OK)),
      "SELECT" => Ok(EventKey(EV_KEY::KEY_SELECT)),
      "GOTO" => Ok(EventKey(EV_KEY::KEY_GOTO)),
      "CLEAR" => Ok(EventKey(EV_KEY::KEY_CLEAR)),
      "POWER2" => Ok(EventKey(EV_KEY::KEY_POWER2)),
      "OPTION" => Ok(EventKey(EV_KEY::KEY_OPTION)),
      "INFO" => Ok(EventKey(EV_KEY::KEY_INFO)),
      "TIME" => Ok(EventKey(EV_KEY::KEY_TIME)),
      "VENDOR" => Ok(EventKey(EV_KEY::KEY_VENDOR)),
      "ARCHIVE" => Ok(EventKey(EV_KEY::KEY_ARCHIVE)),
      "PROGRAM" => Ok(EventKey(EV_KEY::KEY_PROGRAM)),
      "CHANNEL" => Ok(EventKey(EV_KEY::KEY_CHANNEL)),
      "FAVORITES" => Ok(EventKey(EV_KEY::KEY_FAVORITES)),
      "EPG" => Ok(EventKey(EV_KEY::KEY_EPG)),
      "PVR" => Ok(EventKey(EV_KEY::KEY_PVR)),
      "MHP" => Ok(EventKey(EV_KEY::KEY_MHP)),
      "LANGUAGE" => Ok(EventKey(EV_KEY::KEY_LANGUAGE)),
      "TITLE" => Ok(EventKey(EV_KEY::KEY_TITLE)),
      "SUBTITLE" => Ok(EventKey(EV_KEY::KEY_SUBTITLE)),
      "ANGLE" => Ok(EventKey(EV_KEY::KEY_ANGLE)),
      "ZOOM" => Ok(EventKey(EV_KEY::KEY_ZOOM)),
      "MODE" => Ok(EventKey(EV_KEY::KEY_MODE)),
      "KEYBOARD" => Ok(EventKey(EV_KEY::KEY_KEYBOARD)),
      "SCREEN" => Ok(EventKey(EV_KEY::KEY_SCREEN)),
      "PC" => Ok(EventKey(EV_KEY::KEY_PC)),
      "TV" => Ok(EventKey(EV_KEY::KEY_TV)),
      "TV2" => Ok(EventKey(EV_KEY::KEY_TV2)),
      "VCR" => Ok(EventKey(EV_KEY::KEY_VCR)),
      "VCR2" => Ok(EventKey(EV_KEY::KEY_VCR2)),
      "SAT" => Ok(EventKey(EV_KEY::KEY_SAT)),
      "SAT2" => Ok(EventKey(EV_KEY::KEY_SAT2)),
      "CD" => Ok(EventKey(EV_KEY::KEY_CD)),
      "TAPE" => Ok(EventKey(EV_KEY::KEY_TAPE)),
      "RADIO" => Ok(EventKey(EV_KEY::KEY_RADIO)),
      "TUNER" => Ok(EventKey(EV_KEY::KEY_TUNER)),
      "PLAYER" => Ok(EventKey(EV_KEY::KEY_PLAYER)),
      "TEXT" => Ok(EventKey(EV_KEY::KEY_TEXT)),
      "DVD" => Ok(EventKey(EV_KEY::KEY_DVD)),
      "AUX" => Ok(EventKey(EV_KEY::KEY_AUX)),
      "MP3" => Ok(EventKey(EV_KEY::KEY_MP3)),
      "AUDIO" => Ok(EventKey(EV_KEY::KEY_AUDIO)),
      "VIDEO" => Ok(EventKey(EV_KEY::KEY_VIDEO)),
      "DIRECTORY" => Ok(EventKey(EV_KEY::KEY_DIRECTORY)),
      "LIST" => Ok(EventKey(EV_KEY::KEY_LIST)),
      "MEMO" => Ok(EventKey(EV_KEY::KEY_MEMO)),
      "CALENDAR" => Ok(EventKey(EV_KEY::KEY_CALENDAR)),
      "RED" => Ok(EventKey(EV_KEY::KEY_RED)),
      "GREEN" => Ok(EventKey(EV_KEY::KEY_GREEN)),
      "YELLOW" => Ok(EventKey(EV_KEY::KEY_YELLOW)),
      "BLUE" => Ok(EventKey(EV_KEY::KEY_BLUE)),
      "CHANNELUP" => Ok(EventKey(EV_KEY::KEY_CHANNELUP)),
      "CHANNELDOWN" => Ok(EventKey(EV_KEY::KEY_CHANNELDOWN)),
      "FIRST" => Ok(EventKey(EV_KEY::KEY_FIRST)),
      "LAST" => Ok(EventKey(EV_KEY::KEY_LAST)),
      "AB" => Ok(EventKey(EV_KEY::KEY_AB)),
      "NEXT" => Ok(EventKey(EV_KEY::KEY_NEXT)),
      "RESTART" => Ok(EventKey(EV_KEY::KEY_RESTART)),
      "SLOW" => Ok(EventKey(EV_KEY::KEY_SLOW)),
      "SHUFFLE" => Ok(EventKey(EV_KEY::KEY_SHUFFLE)),
      "BREAK" => Ok(EventKey(EV_KEY::KEY_BREAK)),
      "PREVIOUS" => Ok(EventKey(EV_KEY::KEY_PREVIOUS)),
      "DIGITS" => Ok(EventKey(EV_KEY::KEY_DIGITS)),
      "TEEN" => Ok(EventKey(EV_KEY::KEY_TEEN)),
      "TWEN" => Ok(EventKey(EV_KEY::KEY_TWEN)),
      "VIDEOPHONE" => Ok(EventKey(EV_KEY::KEY_VIDEOPHONE)),
      "GAMES" => Ok(EventKey(EV_KEY::KEY_GAMES)),
      "ZOOMIN" => Ok(EventKey(EV_KEY::KEY_ZOOMIN)),
      "ZOOMOUT" => Ok(EventKey(EV_KEY::KEY_ZOOMOUT)),
      "ZOOMRESET" => Ok(EventKey(EV_KEY::KEY_ZOOMRESET)),
      "WORDPROCESSOR" => Ok(EventKey(EV_KEY::KEY_WORDPROCESSOR)),
      "EDITOR" => Ok(EventKey(EV_KEY::KEY_EDITOR)),
      "SPREADSHEET" => Ok(EventKey(EV_KEY::KEY_SPREADSHEET)),
      "GRAPHICSEDITOR" => Ok(EventKey(EV_KEY::KEY_GRAPHICSEDITOR)),
      "PRESENTATION" => Ok(EventKey(EV_KEY::KEY_PRESENTATION)),
      "DATABASE" => Ok(EventKey(EV_KEY::KEY_DATABASE)),
      "NEWS" => Ok(EventKey(EV_KEY::KEY_NEWS)),
      "VOICEMAIL" => Ok(EventKey(EV_KEY::KEY_VOICEMAIL)),
      "ADDRESSBOOK" => Ok(EventKey(EV_KEY::KEY_ADDRESSBOOK)),
      "MESSENGER" => Ok(EventKey(EV_KEY::KEY_MESSENGER)),
      "DISPLAYTOGGLE" => Ok(EventKey(EV_KEY::KEY_DISPLAYTOGGLE)),
      "SPELLCHECK" => Ok(EventKey(EV_KEY::KEY_SPELLCHECK)),
      "LOGOFF" => Ok(EventKey(EV_KEY::KEY_LOGOFF)),
      "DOLLAR" => Ok(EventKey(EV_KEY::KEY_DOLLAR)),
      "EURO" => Ok(EventKey(EV_KEY::KEY_EURO)),
      "FRAMEBACK" => Ok(EventKey(EV_KEY::KEY_FRAMEBACK)),
      "FRAMEFORWARD" => Ok(EventKey(EV_KEY::KEY_FRAMEFORWARD)),
      "CONTEXT_MENU" => Ok(EventKey(EV_KEY::KEY_CONTEXT_MENU)),
      "MEDIA_REPEAT" => Ok(EventKey(EV_KEY::KEY_MEDIA_REPEAT)),
      "10CHANNELSUP" => Ok(EventKey(EV_KEY::KEY_10CHANNELSUP)),
      "10CHANNELSDOWN" => Ok(EventKey(EV_KEY::KEY_10CHANNELSDOWN)),
      "IMAGES" => Ok(EventKey(EV_KEY::KEY_IMAGES)),
      "DEL_EOL" => Ok(EventKey(EV_KEY::KEY_DEL_EOL)),
      "DEL_EOS" => Ok(EventKey(EV_KEY::KEY_DEL_EOS)),
      "INS_LINE" => Ok(EventKey(EV_KEY::KEY_INS_LINE)),
      "DEL_LINE" => Ok(EventKey(EV_KEY::KEY_DEL_LINE)),
      "FN" => Ok(EventKey(EV_KEY::KEY_FN)),
      "FN_ESC" => Ok(EventKey(EV_KEY::KEY_FN_ESC)),
      "FN_F1" => Ok(EventKey(EV_KEY::KEY_FN_F1)),
      "FN_F2" => Ok(EventKey(EV_KEY::KEY_FN_F2)),
      "FN_F3" => Ok(EventKey(EV_KEY::KEY_FN_F3)),
      "FN_F4" => Ok(EventKey(EV_KEY::KEY_FN_F4)),
      "FN_F5" => Ok(EventKey(EV_KEY::KEY_FN_F5)),
      "FN_F6" => Ok(EventKey(EV_KEY::KEY_FN_F6)),
      "FN_F7" => Ok(EventKey(EV_KEY::KEY_FN_F7)),
      "FN_F8" => Ok(EventKey(EV_KEY::KEY_FN_F8)),
      "FN_F9" => Ok(EventKey(EV_KEY::KEY_FN_F9)),
      "FN_F10" => Ok(EventKey(EV_KEY::KEY_FN_F10)),
      "FN_F11" => Ok(EventKey(EV_KEY::KEY_FN_F11)),
      "FN_F12" => Ok(EventKey(EV_KEY::KEY_FN_F12)),
      "FN_1" => Ok(EventKey(EV_KEY::KEY_FN_1)),
      "FN_2" => Ok(EventKey(EV_KEY::KEY_FN_2)),
      "FN_D" => Ok(EventKey(EV_KEY::KEY_FN_D)),
      "FN_E" => Ok(EventKey(EV_KEY::KEY_FN_E)),
      "FN_F" => Ok(EventKey(EV_KEY::KEY_FN_F)),
      "FN_S" => Ok(EventKey(EV_KEY::KEY_FN_S)),
      "FN_B" => Ok(EventKey(EV_KEY::KEY_FN_B)),
      "BRL_DOT1" => Ok(EventKey(EV_KEY::KEY_BRL_DOT1)),
      "BRL_DOT2" => Ok(EventKey(EV_KEY::KEY_BRL_DOT2)),
      "BRL_DOT3" => Ok(EventKey(EV_KEY::KEY_BRL_DOT3)),
      "BRL_DOT4" => Ok(EventKey(EV_KEY::KEY_BRL_DOT4)),
      "BRL_DOT5" => Ok(EventKey(EV_KEY::KEY_BRL_DOT5)),
      "BRL_DOT6" => Ok(EventKey(EV_KEY::KEY_BRL_DOT6)),
      "BRL_DOT7" => Ok(EventKey(EV_KEY::KEY_BRL_DOT7)),
      "BRL_DOT8" => Ok(EventKey(EV_KEY::KEY_BRL_DOT8)),
      "BRL_DOT9" => Ok(EventKey(EV_KEY::KEY_BRL_DOT9)),
      "BRL_DOT10" => Ok(EventKey(EV_KEY::KEY_BRL_DOT10)),
      "NUMERIC_0" => Ok(EventKey(EV_KEY::KEY_NUMERIC_0)),
      "NUMERIC_1" => Ok(EventKey(EV_KEY::KEY_NUMERIC_1)),
      "NUMERIC_2" => Ok(EventKey(EV_KEY::KEY_NUMERIC_2)),
      "NUMERIC_3" => Ok(EventKey(EV_KEY::KEY_NUMERIC_3)),
      "NUMERIC_4" => Ok(EventKey(EV_KEY::KEY_NUMERIC_4)),
      "NUMERIC_5" => Ok(EventKey(EV_KEY::KEY_NUMERIC_5)),
      "NUMERIC_6" => Ok(EventKey(EV_KEY::KEY_NUMERIC_6)),
      "NUMERIC_7" => Ok(EventKey(EV_KEY::KEY_NUMERIC_7)),
      "NUMERIC_8" => Ok(EventKey(EV_KEY::KEY_NUMERIC_8)),
      "NUMERIC_9" => Ok(EventKey(EV_KEY::KEY_NUMERIC_9)),
      "NUMERIC_STAR" => Ok(EventKey(EV_KEY::KEY_NUMERIC_STAR)),
      "NUMERIC_POUND" => Ok(EventKey(EV_KEY::KEY_NUMERIC_POUND)),
      "NUMERIC_A" => Ok(EventKey(EV_KEY::KEY_NUMERIC_A)),
      "NUMERIC_B" => Ok(EventKey(EV_KEY::KEY_NUMERIC_B)),
      "NUMERIC_C" => Ok(EventKey(EV_KEY::KEY_NUMERIC_C)),
      "NUMERIC_D" => Ok(EventKey(EV_KEY::KEY_NUMERIC_D)),
      "CAMERA_FOCUS" => Ok(EventKey(EV_KEY::KEY_CAMERA_FOCUS)),
      "WPS_BUTTON" => Ok(EventKey(EV_KEY::KEY_WPS_BUTTON)),
      "TOUCHPAD_TOGGLE" => Ok(EventKey(EV_KEY::KEY_TOUCHPAD_TOGGLE)),
      "TOUCHPAD_ON" => Ok(EventKey(EV_KEY::KEY_TOUCHPAD_ON)),
      "TOUCHPAD_OFF" => Ok(EventKey(EV_KEY::KEY_TOUCHPAD_OFF)),
      "CAMERA_ZOOMIN" => Ok(EventKey(EV_KEY::KEY_CAMERA_ZOOMIN)),
      "CAMERA_ZOOMOUT" => Ok(EventKey(EV_KEY::KEY_CAMERA_ZOOMOUT)),
      "CAMERA_UP" => Ok(EventKey(EV_KEY::KEY_CAMERA_UP)),
      "CAMERA_DOWN" => Ok(EventKey(EV_KEY::KEY_CAMERA_DOWN)),
      "CAMERA_LEFT" => Ok(EventKey(EV_KEY::KEY_CAMERA_LEFT)),
      "CAMERA_RIGHT" => Ok(EventKey(EV_KEY::KEY_CAMERA_RIGHT)),
      "ATTENDANT_ON" => Ok(EventKey(EV_KEY::KEY_ATTENDANT_ON)),
      "ATTENDANT_OFF" => Ok(EventKey(EV_KEY::KEY_ATTENDANT_OFF)),
      "ATTENDANT_TOGGLE" => Ok(EventKey(EV_KEY::KEY_ATTENDANT_TOGGLE)),
      "LIGHTS_TOGGLE" => Ok(EventKey(EV_KEY::KEY_LIGHTS_TOGGLE)),
      "ALS_TOGGLE" => Ok(EventKey(EV_KEY::KEY_ALS_TOGGLE)),
      "ROTATE_LOCK_TOGGLE" => Ok(EventKey(EV_KEY::KEY_ROTATE_LOCK_TOGGLE)),
      "BUTTONCONFIG" => Ok(EventKey(EV_KEY::KEY_BUTTONCONFIG)),
      "TASKMANAGER" => Ok(EventKey(EV_KEY::KEY_TASKMANAGER)),
      "JOURNAL" => Ok(EventKey(EV_KEY::KEY_JOURNAL)),
      "CONTROLPANEL" => Ok(EventKey(EV_KEY::KEY_CONTROLPANEL)),
      "APPSELECT" => Ok(EventKey(EV_KEY::KEY_APPSELECT)),
      "SCREENSAVER" => Ok(EventKey(EV_KEY::KEY_SCREENSAVER)),
      "VOICECOMMAND" => Ok(EventKey(EV_KEY::KEY_VOICECOMMAND)),
      "ASSISTANT" => Ok(EventKey(EV_KEY::KEY_ASSISTANT)),
      "BRIGHTNESS_MIN" => Ok(EventKey(EV_KEY::KEY_BRIGHTNESS_MIN)),
      "BRIGHTNESS_MAX" => Ok(EventKey(EV_KEY::KEY_BRIGHTNESS_MAX)),
      "KBDINPUTASSIST_PREV" => Ok(EventKey(EV_KEY::KEY_KBDINPUTASSIST_PREV)),
      "KBDINPUTASSIST_NEXT" => Ok(EventKey(EV_KEY::KEY_KBDINPUTASSIST_NEXT)),
      "KBDINPUTASSIST_PREVGROUP" => Ok(EventKey(EV_KEY::KEY_KBDINPUTASSIST_PREVGROUP)),
      "KBDINPUTASSIST_NEXTGROUP" => Ok(EventKey(EV_KEY::KEY_KBDINPUTASSIST_NEXTGROUP)),
      "KBDINPUTASSIST_ACCEPT" => Ok(EventKey(EV_KEY::KEY_KBDINPUTASSIST_ACCEPT)),
      "KBDINPUTASSIST_CANCEL" => Ok(EventKey(EV_KEY::KEY_KBDINPUTASSIST_CANCEL)),
      "RIGHT_UP" => Ok(EventKey(EV_KEY::KEY_RIGHT_UP)),
      "RIGHT_DOWN" => Ok(EventKey(EV_KEY::KEY_RIGHT_DOWN)),
      "LEFT_UP" => Ok(EventKey(EV_KEY::KEY_LEFT_UP)),
      "LEFT_DOWN" => Ok(EventKey(EV_KEY::KEY_LEFT_DOWN)),
      "ROOT_MENU" => Ok(EventKey(EV_KEY::KEY_ROOT_MENU)),
      "MEDIA_TOP_MENU" => Ok(EventKey(EV_KEY::KEY_MEDIA_TOP_MENU)),
      "NUMERIC_11" => Ok(EventKey(EV_KEY::KEY_NUMERIC_11)),
      "NUMERIC_12" => Ok(EventKey(EV_KEY::KEY_NUMERIC_12)),
      "AUDIO_DESC" => Ok(EventKey(EV_KEY::KEY_AUDIO_DESC)),
      "3D_MODE" => Ok(EventKey(EV_KEY::KEY_3D_MODE)),
      "NEXT_FAVORITE" => Ok(EventKey(EV_KEY::KEY_NEXT_FAVORITE)),
      "STOP_RECORD" => Ok(EventKey(EV_KEY::KEY_STOP_RECORD)),
      "PAUSE_RECORD" => Ok(EventKey(EV_KEY::KEY_PAUSE_RECORD)),
      "VOD" => Ok(EventKey(EV_KEY::KEY_VOD)),
      "UNMUTE" => Ok(EventKey(EV_KEY::KEY_UNMUTE)),
      "FASTREVERSE" => Ok(EventKey(EV_KEY::KEY_FASTREVERSE)),
      "SLOWREVERSE" => Ok(EventKey(EV_KEY::KEY_SLOWREVERSE)),
      "DATA" => Ok(EventKey(EV_KEY::KEY_DATA)),
      "ONSCREEN_KEYBOARD" => Ok(EventKey(EV_KEY::KEY_ONSCREEN_KEYBOARD)),
      "MAX" => Ok(EventKey(EV_KEY::KEY_MAX)),
      value => Err(serde::de::Error::unknown_variant(value, &["keycode name"])),
    }
  }
}
