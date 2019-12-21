use crate::config::*;
use evdev_rs::enums::EV_KEY;
use maplit::btreeset;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::convert::{TryFrom, TryInto};
use std::ops::Deref;

// Remaps Event to Vec<Event>
pub(crate) struct Remapper {
  keymap: &'static Vec<Rule>,
  active_rules: BTreeSet<&'static Rule>,
  active_passthrus: BTreeSet<EventKey>,
  last_key: EventKey,
}

impl Remapper {
  pub(crate) fn new(keymap: &'static Vec<Rule>) -> Self {
    Self {
      keymap,
      active_rules: BTreeSet::new(),
      active_passthrus: BTreeSet::new(),
      last_key: EV_KEY::KEY_UNKNOWN.into(),
    }
  }

  pub(crate) fn remap(&mut self, received: Event) -> BTreeSet<Event> {
    let old_virtually_pressed = self.virtually_pressed();

    self.add_remove_actives(&received);
    self.convert_actives();

    let mut to_be_sent = BTreeSet::new();
    to_be_sent.extend(self.events_for_diff(&old_virtually_pressed));
    to_be_sent.extend(self.events_for_keyrepeats(received));

    to_be_sent
  }

  fn add_remove_actives(&mut self, received: &Event) {
    match received.event_type {
      EventType::Press => {
        self.active_passthrus.insert(received.key.clone());
      }
      EventType::Release => {
        self.active_passthrus.remove(&received.key);

        let pressed = self
          .actually_pressed()
          .difference(&btreeset![received.key.clone()])
          .cloned()
          .collect();
        for rule in self.active_rules.clone().into_iter() {
          if !self.is_active(rule, &pressed) {
            self.active_rules.remove(rule);
          }
        }
      }
      EventType::Repeat => {}
    }
  }

  fn convert_actives(&mut self) {
    let mut already_handled_keys: BTreeSet<EventKey> = Default::default();

    for rule in self.keymap.iter() {
      for passthru in self.active_passthrus.clone().into_iter() {
        if !already_handled_keys.contains(&passthru)
          && self.is_active(rule, &btreeset![passthru.clone()])
        {
          self.active_passthrus.remove(&passthru);
          self.active_rules.insert(rule);
          already_handled_keys.insert(passthru);
        }
      }
    }

    for rule in self.active_rules.clone().into_iter() {
      if !self.is_active(rule, &self.actually_pressed()) {
        self.active_rules.remove(rule);
        self.active_passthrus.insert(rule.from.key.clone());
      }
    }
  }

  fn events_for_diff(&self, old_virtually_pressed: &BTreeSet<EventKey>) -> BTreeSet<Event> {
    let mut result = BTreeSet::new();

    for press in self.virtually_pressed().difference(old_virtually_pressed) {
      result.insert(Event {
        event_type: EventType::Press,
        key: press.clone(),
      });
    }

    for release in old_virtually_pressed.difference(&self.virtually_pressed()) {
      result.insert(Event {
        event_type: EventType::Release,
        key: release.clone(),
      });
    }

    result
  }

  fn events_for_keyrepeats(&self, received: Event) -> Option<Event> {
    if received.event_type != EventType::Repeat {
      return None;
    }

    for rule in self.active_rules.iter() {
      if self.is_active(rule, &btreeset![received.key.clone()]) {
        return Some(Event {
          event_type: EventType::Repeat,
          key: rule.to.key.clone(),
        });
      }
    }

    Some(received)
  }

  fn actually_pressed(&self) -> BTreeSet<EventKey> {
    self
      .active_rules
      .iter()
      .map(|rule| rule.from.key.clone())
      .chain(self.active_passthrus.iter().cloned())
      .collect()
  }

  fn is_active(&self, rule: &'static Rule, pressed: &BTreeSet<EventKey>) -> bool {
    let remapped_modifiers: BTreeSet<Modifier> = self
      .active_rules
      .iter()
      .map(|rule| rule.to.key.clone())
      .chain(self.active_passthrus.iter().cloned())
      .filter_map(|key| key.try_into().ok())
      .collect();

    pressed.contains(&rule.from.key)
      && rule
        .from
        .with
        .as_ref()
        .map(|config_modifiers| remapped_modifiers.is_superset(&config_modifiers))
        .unwrap_or(true)
      && rule
        .from
        .without
        .as_ref()
        .map(|config_modifiers| remapped_modifiers.is_disjoint(&config_modifiers))
        .unwrap_or(true)
  }

  fn virtually_pressed(&self) -> BTreeSet<EventKey> {
    let empty = BTreeSet::new();
    let mut result = self.active_passthrus.clone();

    for rule in self.active_rules.iter() {
      result.insert(rule.to.key.clone());
    }

    for rule in self.active_rules.iter() {
      for modifier in rule.from.with.as_ref().unwrap_or(&empty).iter() {
        result.remove(&modifier.into());
      }
    }

    for rule in self.active_rules.iter() {
      result.insert(rule.to.key.clone());

      for modifier in rule.to.with.as_ref().unwrap_or(&empty).iter() {
        result.insert(modifier.into());
      }
    }

    result
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Event {
  pub(crate) event_type: EventType,
  pub(crate) key: EventKey,
}

impl Ord for Event {
  // events for modifier keys are smaller
  // cuz those events need to get sent first
  fn cmp(&self, other: &Self) -> Ordering {
    let modifier1: Option<Modifier> = (&self.key).try_into().ok();
    let modifier2: Option<Modifier> = (&other.key).try_into().ok();

    match (modifier1, modifier2) {
      (Some(_), None) => Ordering::Less,
      (None, Some(_)) => Ordering::Greater,
      _ => (self.key.clone(), self.event_type).cmp(&(other.key.clone(), self.event_type)),
    }
  }
}

impl PartialOrd for Event {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum EventType {
  Press,
  Release,
  Repeat,
}

impl TryFrom<i32> for EventType {
  type Error = ();

  fn try_from(value: i32) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Self::Release),
      1 => Ok(Self::Press),
      2 => Ok(Self::Repeat),
      _ => Err(()),
    }
  }
}

impl Into<i32> for EventType {
  fn into(self) -> i32 {
    match self {
      Self::Release => 0,
      Self::Press => 1,
      Self::Repeat => 2,
    }
  }
}

impl TryFrom<EventKey> for Modifier {
  type Error = ();

  fn try_from(key: EventKey) -> Result<Self, Self::Error> {
    (&key).try_into()
  }
}

impl TryFrom<&EventKey> for Modifier {
  type Error = ();

  fn try_from(key: &EventKey) -> Result<Self, Self::Error> {
    match key.deref() {
      EV_KEY::KEY_LEFTSHIFT => Ok(Self::LEFTSHIFT),
      EV_KEY::KEY_RIGHTSHIFT => Ok(Self::RIGHTSHIFT),
      EV_KEY::KEY_LEFTCTRL => Ok(Self::LEFTCTRL),
      EV_KEY::KEY_RIGHTCTRL => Ok(Self::RIGHTCTRL),
      EV_KEY::KEY_LEFTALT => Ok(Self::LEFTALT),
      EV_KEY::KEY_RIGHTALT => Ok(Self::RIGHTALT),
      EV_KEY::KEY_LEFTMETA => Ok(Self::LEFTMETA),
      EV_KEY::KEY_RIGHTMETA => Ok(Self::RIGHTMETA),
      _ => Err(()),
    }
  }
}

impl Into<EventKey> for Modifier {
  fn into(self) -> EventKey {
    (&self).into()
  }
}

impl Into<EventKey> for &Modifier {
  fn into(self) -> EventKey {
    match self {
      Modifier::LEFTSHIFT => EV_KEY::KEY_LEFTSHIFT.into(),
      Modifier::RIGHTSHIFT => EV_KEY::KEY_RIGHTSHIFT.into(),
      Modifier::LEFTCTRL => EV_KEY::KEY_LEFTCTRL.into(),
      Modifier::RIGHTCTRL => EV_KEY::KEY_RIGHTCTRL.into(),
      Modifier::LEFTALT => EV_KEY::KEY_LEFTALT.into(),
      Modifier::RIGHTALT => EV_KEY::KEY_RIGHTALT.into(),
      Modifier::LEFTMETA => EV_KEY::KEY_LEFTMETA.into(),
      Modifier::RIGHTMETA => EV_KEY::KEY_RIGHTMETA.into(),
    }
  }
}
