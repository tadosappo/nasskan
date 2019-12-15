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
  active_keys: BTreeSet<KeyState>,
  last_key: EventKey,
}

impl Remapper {
  pub(crate) fn new(keymap: &'static Vec<Rule>) -> Self {
    Self {
      keymap,
      active_keys: BTreeSet::new(),
      last_key: EV_KEY::KEY_UNKNOWN.into(),
    }
  }

  pub(crate) fn remap(&mut self, event: Event) -> BTreeSet<Event> {
    let old_active_keys = self.active_keys.clone();
    self.add_remove_active_key(&event);
    self.update_each_active_key();

    let mut events: BTreeSet<Event> = BTreeSet::new();
    events.extend(self.events_for_appeared_key(&old_active_keys));
    events.extend(self.events_for_disappeared_key(&old_active_keys));

    if let Some(keyrepeat) = self.keyrepeat(&event, &old_active_keys) {
      events.insert(keyrepeat);
    }

    events
  }

  fn add_remove_active_key(&mut self, received: &Event) {
    match received.event_type {
      EventType::Press => {
        self
          .active_keys
          .insert(KeyState::Passthru(received.key.clone()));
      }
      EventType::Release => {
        self
          .active_keys
          .remove(&KeyState::Passthru(received.key.clone()));

        for rule in self.active_remaps() {
          if received.key == rule.from.key {
            self.active_keys.remove(&KeyState::Remapped(rule));
          }
        }
      }
      EventType::Repeat => {}
    }
  }

  fn update_each_active_key(&mut self) {
    let remaps = self.active_remaps();
    let passthrus = self.active_passthrus();

    for config_rule in self.keymap.iter() {
      let remapped_modifiers = self.remapped_modifiers();

      for rule in remaps.iter() {
        if !config_rule.is_active(&rule.from.key, &remapped_modifiers) {
          self.active_keys.remove(&KeyState::Remapped(config_rule));
          self
            .active_keys
            .insert(KeyState::Passthru(rule.from.key.clone()));
        }
      }

      for passthru in passthrus.iter() {
        if config_rule.is_active(passthru, &remapped_modifiers) {
          self
            .active_keys
            .remove(&KeyState::Passthru(passthru.clone()));
          self.active_keys.insert(KeyState::Remapped(config_rule));
        }
      }
    }
  }

  fn remapped_modifiers(&self) -> BTreeSet<Modifier> {
    self
      .active_keys
      .iter()
      .filter_map(|key| key.remapped_key().clone().try_into().ok())
      .collect()
  }

  fn active_passthrus(&self) -> BTreeSet<EventKey> {
    self
      .active_keys
      .iter()
      .filter_map(|key| match key {
        KeyState::Passthru(key) => Some(key.clone()),
        KeyState::Remapped(_) => None,
      })
      .collect()
  }

  fn active_remaps(&self) -> BTreeSet<&'static Rule> {
    self
      .active_keys
      .iter()
      .filter_map(|key| match key {
        KeyState::Passthru(_) => None,
        KeyState::Remapped(rule) => Some(*rule),
      })
      .collect()
  }

  fn events_for_appeared_key(&self, old_active_keys: &BTreeSet<KeyState>) -> BTreeSet<Event> {
    self
      .active_keys
      .difference(old_active_keys)
      .flat_map(|appeared_key| appeared_key.events(EventType::Press))
      .collect()
  }

  fn events_for_disappeared_key(&self, old_active_keys: &BTreeSet<KeyState>) -> BTreeSet<Event> {
    old_active_keys
      .difference(&self.active_keys)
      .flat_map(|disappeared_key| disappeared_key.events(EventType::Release))
      .collect()
  }

  fn keyrepeat(&self, event: &Event, old_active_keys: &BTreeSet<KeyState>) -> Option<Event> {
    let remapped_modifiers = self.remapped_modifiers();

    if self
      .active_keys
      .contains(&KeyState::Passthru(event.key.clone()))
      && old_active_keys.contains(&KeyState::Passthru(event.key.clone()))
    {
      return Some(Event {
        event_type: EventType::Repeat,
        key: event.key.clone(),
      });
    }

    for key in self.active_keys.intersection(&old_active_keys) {
      match key {
        KeyState::Passthru(_) => {}
        KeyState::Remapped(rule) => {
          if rule.is_active(&event.key, &remapped_modifiers) {
            return Some(Event {
              event_type: EventType::Repeat,
              key: rule.to.key.clone(),
            });
          }
        }
      }
    }

    None
  }
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
enum KeyState {
  Passthru(EventKey),
  Remapped(&'static Rule),
}

impl KeyState {
  fn original_key(&self) -> &EventKey {
    match self {
      KeyState::Passthru(key) => key,
      KeyState::Remapped(rule) => &rule.from.key,
    }
  }

  fn remapped_key(&self) -> &EventKey {
    match self {
      KeyState::Passthru(key) => key,
      KeyState::Remapped(rule) => &rule.to.key,
    }
  }

  fn remapped_keys(&self) -> BTreeSet<EventKey> {
    let empty = BTreeSet::new();
    let mut result = btreeset![self.remapped_key().clone()];

    match self {
      KeyState::Passthru(_) => {}
      KeyState::Remapped(rule) => {
        for modifier in rule.to.with.as_ref().unwrap_or(&empty).iter() {
          let keys: Vec<EventKey> = modifier.into();
          result.extend(keys);
        }
      }
    }

    result
  }

  fn events(&self, event_type: EventType) -> BTreeSet<Event> {
    match self {
      KeyState::Passthru(key) => btreeset![Event {
        event_type,
        key: key.clone(),
      }],
      KeyState::Remapped(rule) => {
        let empty = BTreeSet::new();
        let modifiers = rule
          .to
          .with
          .as_ref()
          .unwrap_or(&empty)
          .into_iter()
          .flat_map(|modifier| {
            let keys: Vec<EventKey> = modifier.into();
            keys.into_iter().map(|key| Event { event_type, key })
          });
        let key = Event {
          event_type,
          key: rule.to.key.clone(),
        };
        std::iter::once(key).chain(modifiers).collect()
      }
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
      _ => (self.event_type, self.key.clone()).cmp(&(other.event_type, other.key.clone())),
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
  Release,
  Press,
  Repeat,
}

impl EventType {
  fn invert(&self) -> Self {
    match self {
      EventType::Release => EventType::Press,
      EventType::Press => EventType::Release,
      EventType::Repeat => EventType::Repeat,
    }
  }
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

impl Rule {
  pub(crate) fn is_active(&self, key: &EventKey, remapped_modifiers: &BTreeSet<Modifier>) -> bool {
    key == &self.from.key
      && self
        .from
        .with
        .as_ref()
        .map(|config_modifiers| remapped_modifiers.is_superset(&config_modifiers))
        .unwrap_or(true)
      && self
        .from
        .without
        .as_ref()
        .map(|config_modifiers| remapped_modifiers.is_disjoint(&config_modifiers))
        .unwrap_or(true)
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
      EV_KEY::KEY_LEFTSHIFT | EV_KEY::KEY_RIGHTSHIFT => Ok(Self::Shift),
      EV_KEY::KEY_LEFTCTRL | EV_KEY::KEY_RIGHTCTRL => Ok(Self::Control),
      EV_KEY::KEY_LEFTALT | EV_KEY::KEY_RIGHTALT => Ok(Self::Alt),
      EV_KEY::KEY_LEFTMETA | EV_KEY::KEY_RIGHTMETA => Ok(Self::Meta),
      _ => Err(()),
    }
  }
}

impl Into<Vec<EventKey>> for Modifier {
  fn into(self) -> Vec<EventKey> {
    (&self).into()
  }
}

impl Into<Vec<EventKey>> for &Modifier {
  fn into(self) -> Vec<EventKey> {
    match self {
      Modifier::Shift => vec![EV_KEY::KEY_LEFTSHIFT.into(), EV_KEY::KEY_RIGHTSHIFT.into()],
      Modifier::Control => vec![EV_KEY::KEY_LEFTCTRL.into(), EV_KEY::KEY_RIGHTCTRL.into()],
      Modifier::Alt => vec![EV_KEY::KEY_LEFTALT.into(), EV_KEY::KEY_RIGHTALT.into()],
      Modifier::Meta => vec![EV_KEY::KEY_LEFTMETA.into(), EV_KEY::KEY_RIGHTMETA.into()],
    }
  }
}
