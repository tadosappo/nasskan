use crate::config::*;
use evdev_rs::enums::EV_KEY;
use maplit::btreeset;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{TryFrom, TryInto};
use std::ops::Deref;

// Remaps Event to Vec<Event>
pub(crate) struct Remapper {
  keymap: &'static Vec<Rule>,
  keyboard_state: Vec<KeyState>,
  last_key: EventKey,
}

impl Remapper {
  pub(crate) fn new(keymap: &'static Vec<Rule>) -> Self {
    Self {
      keymap,
      keyboard_state: Vec::new(),
      last_key: EV_KEY::KEY_RESERVED.into(),
    }
  }

  pub(crate) fn remap(&mut self, received: Event) -> BTreeSet<Event> {
    let old_virtually_pressed = self.virtually_pressed();

    self.add_remove_actives(&received);
    self.convert_actives();

    let mut to_be_sent = BTreeSet::new();
    to_be_sent.extend(self.events_for_diff(&old_virtually_pressed));
    to_be_sent.extend(self.events_for_tap(&received));
    to_be_sent.extend(self.events_for_keyrepeats(received.clone()));
    self.last_key = received.key.clone();

    to_be_sent
  }

  fn add_remove_actives(&mut self, received: &Event) {
    let empty = BTreeSet::new();
    let modifier_map = self.modifier_map();
    let remapped_modifier = modifier_map.get(&received.key);

    match received.event_type {
      EventType::Press => {
        self
          .keyboard_state
          .push(KeyState::Passthru(received.key.clone()));
      }
      EventType::Release => self.keyboard_state.retain(|key_state| match key_state {
        KeyState::Passthru(key) => key != &received.key,
        KeyState::Remapped(rule) => {
          rule.from.key != received.key
            && remapped_modifier
              .map(|remapped_modifier| {
                !rule
                  .from
                  .with
                  .as_ref()
                  .unwrap_or(&empty)
                  .contains(remapped_modifier)
              })
              .unwrap_or(true)
        }
      }),
      EventType::Repeat => {}
    }
  }

  fn convert_actives(&mut self) {
    let original_keys: Vec<EventKey> = self
      .keyboard_state
      .iter()
      .map(|key_state| key_state.original_key())
      .collect();

    // Initialize modifier state
    for key_state in self.keyboard_state.iter_mut() {
      *key_state = KeyState::Passthru(EV_KEY::KEY_RESERVED.into())
    }

    for config_rule in self.keymap.iter() {
      for (i, original_key) in original_keys.iter().enumerate() {
        if let KeyState::Remapped(_) = self.keyboard_state[i] {
          continue;
        }

        if self.is_active(config_rule, &original_key) {
          std::mem::replace(&mut self.keyboard_state[i], KeyState::Remapped(config_rule));
          break;
        }
      }
    }

    for (i, key_state) in self.keyboard_state.iter_mut().enumerate() {
      if key_state == &KeyState::Passthru(EV_KEY::KEY_RESERVED.into()) {
        *key_state = KeyState::Passthru(original_keys[i].clone());
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

    for rule in self.active_rules() {
      if received.key == rule.from.key {
        return Some(Event {
          event_type: EventType::Repeat,
          key: rule.to.key.clone(),
        });
      }
    }

    Some(received)
  }

  fn events_for_tap(&self, received: &Event) -> BTreeSet<Event> {
    if received.event_type != EventType::Release {
      return BTreeSet::new();
    }

    if self.last_key != received.key {
      return BTreeSet::new();
    }

    for rule in self.keymap.iter() {
      if let Some(tap) = &rule.tap {
        if received.key == rule.from.key {
          return btreeset![
            Event {
              event_type: EventType::Press,
              key: tap.key.clone()
            },
            Event {
              event_type: EventType::Release,
              key: tap.key.clone()
            }
          ];
        }
      }
    }

    BTreeSet::new()
  }

  fn is_active(&self, rule: &'static Rule, pressed: &EventKey) -> bool {
    let remapped_modifiers: BTreeSet<Modifier> = self
      .keyboard_state
      .iter()
      .map(|key_state| key_state.remapped_key())
      .filter_map(|key| key.try_into().ok())
      .collect();

    pressed == &rule.from.key
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
    let mut result: BTreeSet<EventKey> = self
      .keyboard_state
      .iter()
      .map(|key_state| key_state.remapped_key())
      .collect();

    if let Some(last_key_state) = self.keyboard_state.last() {
      if let KeyState::Remapped(last_rule) = last_key_state {
        for modifier in last_rule.from.with.as_ref().unwrap_or(&empty).iter() {
          result.remove(&modifier.into());
        }

        result.insert(last_rule.to.key.clone());

        for modifier in last_rule.to.with.as_ref().unwrap_or(&empty).iter() {
          result.insert(modifier.into());
        }
      }
    }

    result
  }

  fn active_rules<'a>(&'a self) -> impl Iterator<Item = &'static Rule> + 'a {
    self
      .keyboard_state
      .iter()
      .filter_map(|key_state| match key_state {
        KeyState::Passthru(_) => None,
        KeyState::Remapped(rule) => Some(*rule),
      })
  }

  fn modifier_map(&self) -> BTreeMap<EventKey, Modifier> {
    let mut result = BTreeMap::new();

    for rule in self.keymap.iter() {
      if let Some(modifier) = (&rule.to.key).try_into().ok() {
        result.insert(rule.from.key.clone(), modifier);
      }
    }

    result
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum KeyState {
  Passthru(EventKey),
  Remapped(&'static Rule),
}

impl KeyState {
  fn original_key(&self) -> EventKey {
    match self {
      KeyState::Passthru(passthru) => passthru.clone(),
      KeyState::Remapped(rule) => rule.from.key.clone(),
    }
  }

  fn remapped_key(&self) -> EventKey {
    match self {
      KeyState::Passthru(passthru) => passthru.clone(),
      KeyState::Remapped(rule) => rule.to.key.clone(),
    }
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
      _ => (self.key.clone(), self.event_type).cmp(&(other.key.clone(), other.event_type)),
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
