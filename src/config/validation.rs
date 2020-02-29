use super::{Config, Modifier};
use std::convert::TryInto;

pub(crate) fn validate_order(config: &Config) {
  for device in config.devices.iter() {
    let mut key_found = false;
    for rule in device.then.iter() {
      if ((&rule.to.key).try_into().ok() as Option<Modifier>).is_none() {
        key_found = true;
      } else if key_found {
        panic!("Remap rules for modifiers should be at first");
      }
    }
  }
}

pub(crate) fn validate_tap(config: &Config) {
  for device in config.devices.iter() {
    for rule in device.then.iter() {
      if rule.tap.is_some()
        && (rule
          .from
          .with
          .as_ref()
          .map(|modifiers| 0 < modifiers.len())
          .unwrap_or(false)
          || rule
            .from
            .without
            .as_ref()
            .map(|modifiers| 0 < modifiers.len())
            .unwrap_or(false)
          || rule
            .to
            .with
            .as_ref()
            .map(|modifiers| 0 < modifiers.len())
            .unwrap_or(false))
      {
        panic!("Remap rules with tap should not have from.with, from.without or to.with clause");
      }
    }
  }
}
