use evdev_rs as evdev;
use log::*;
use mio::unix::EventedFd;
use mio::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;

mod remapper;
use remapper::*;
mod config;
use config::*;

trait AsyncWorker: AsRawFd {
  fn step(&mut self, manager: &mut WorkerManager);
}

struct WorkerManager {
  poll: mio::Poll,
  workers: BTreeMap<usize, Rc<RefCell<dyn AsyncWorker>>>,
}

impl WorkerManager {
  fn new() -> Self {
    Self {
      poll: Poll::new().unwrap(),
      workers: BTreeMap::new(),
    }
  }

  fn run(&mut self) {
    let mut events = Events::with_capacity(128);
    loop {
      self.poll.poll(&mut events, None).unwrap();

      for event in events.iter() {
        if let Some(worker) = self.workers.get_mut(&event.token().0) {
          Rc::clone(worker).borrow_mut().step(self);
        }
        // The reason why I use Vec<Rc<RefCell<AsyncWorker>>> instead of Vec<Box<AsyncWorker>> is:
        // While doing `worker.step(manager)`, there's a possibility for `step` to remove `worker` itself using `manager`.
        // It took a while to figure out what cryptic messages from borrow checker means...
      }
    }
  }

  fn start<T: AsyncWorker + 'static>(&mut self, id: usize, worker: T) {
    self
      .poll
      .register(
        &EventedFd(&worker.as_raw_fd()),
        Token(id),
        Ready::readable(),
        PollOpt::edge(),
      )
      .unwrap();

    self.workers.insert(id, Rc::new(RefCell::new(worker)));
  }

  fn stop(&mut self, id: usize) {
    match self.workers.remove(&id) {
      Some(worker) => self
        .poll
        .deregister(&EventedFd(&worker.borrow().as_raw_fd()))
        .unwrap(),
      None => return,
    }
  }
}

struct KeyboardConnectionWorker {
  monitor: udev::MonitorSocket,
}

impl KeyboardConnectionWorker {
  fn new(ctx: &udev::Context) -> Self {
    let mut builder = udev::MonitorBuilder::new(ctx).unwrap();
    builder.match_subsystem("input").unwrap();
    let monitor = builder.listen().unwrap();

    Self { monitor }
  }
}

impl AsRawFd for KeyboardConnectionWorker {
  fn as_raw_fd(&self) -> RawFd {
    self.monitor.as_raw_fd()
  }
}

impl AsyncWorker for KeyboardConnectionWorker {
  fn step(&mut self, manager: &mut WorkerManager) {
    loop {
      let event = match self.monitor.next() {
        Some(event) => event,
        None => return,
      };

      let connected_device = event.device();
      let device_id = match connected_device.devnum() {
        Some(devnum) => devnum,
        None => return,
      };

      match event.event_type() {
        udev::EventType::Add => {
          if let Some(worker) = KeyPressWorker::for_keyboard(&connected_device) {
            info!("keyboard {} connected", device_id);
            manager.start(device_id.try_into().unwrap(), worker);
          }
        }
        udev::EventType::Remove => {
          info!("keyboard {} disconnected", device_id);
          manager.stop(device_id.try_into().unwrap());
        }
        _ => {}
      }
    }
  }
}

struct KeyPressWorker {
  actual_keyboard: evdev::Device,
  virtual_keyboard: evdev::UInputDevice,
  remapper: Remapper,
}

impl KeyPressWorker {
  fn new(path: &std::path::Path, remapper: Remapper) -> Result<Self, nix::errno::Errno> {
    let file = OpenOptions::new()
      .read(true)
      .custom_flags(nix::fcntl::OFlag::O_NONBLOCK.bits())
      .open(path)
      .unwrap();
    let mut actual_keyboard = evdev::Device::new_from_fd(file)?;
    let virtual_keyboard = evdev::UInputDevice::create_from_device(&actual_keyboard)
      .expect("Creating uinput device failed. Maybe uinput kernel module is not loaded?");
    actual_keyboard.grab(evdev::GrabMode::Grab).expect("Some process have grabbed this keyboard already. Maybe there's an another instance of nasskan running?");

    Ok(Self {
      actual_keyboard,
      virtual_keyboard,
      remapper,
    })
  }

  fn for_keyboard(keyboard: &udev::Device) -> Option<Self> {
    let device_file_path = match keyboard.devnode() {
      Some(devnode) => devnode,
      None => return None,
    };

    for config_device in CONFIG.devices.iter() {
      let is_connected = config_device
        .if_
        .iter()
        .all(|(name, value)| keyboard.property_value(name).and_then(|x| x.to_str()) == Some(value));

      if is_connected {
        let remapper = Remapper::new(&config_device.then);
        if let Ok(x) = KeyPressWorker::new(device_file_path, remapper) {
          return Some(x);
        };

        warn!("evdev_new_from_fd failed. It means some non-keyboard device matched with devices.if clause in your config. You should write it more strictly")
      }
    }

    None
  }

  fn handle_event(&mut self, input_event: evdev::InputEvent) {
    let key: EventKey = match &input_event.event_code {
      evdev::enums::EventCode::EV_KEY(ref key) => {
        trace!("Received an evdev event: {:?}", input_event);
        key.clone().into()
      }
      _ => {
        trace!("Ignored an evdev event: {:?}", input_event);
        return;
      }
    };
    let event_type: EventType = input_event
      .value
      .try_into()
      .expect("an evdev event has invalid value");
    let event = remapper::Event { event_type, key };

    debug!("Input: {:?}", event);
    let remapped_events = self.remapper.remap(event);
    debug!("Output: {:?}", remapped_events);

    for remapped_event in remapped_events {
      self
        .virtual_keyboard
        .write_event(&evdev::InputEvent::new(
          &input_event.time,
          &evdev::enums::EventCode::EV_KEY(remapped_event.key.into()),
          remapped_event.event_type.into(),
        ))
        .unwrap();
    }

    self
      .virtual_keyboard
      .write_event(&evdev::InputEvent {
        event_type: evdev::enums::EventType::EV_SYN,
        event_code: evdev::enums::EventCode::EV_SYN(evdev::enums::EV_SYN::SYN_REPORT),
        value: 0,
        ..input_event
      })
      .unwrap();
  }
}

impl AsRawFd for KeyPressWorker {
  fn as_raw_fd(&self) -> RawFd {
    let file = self.actual_keyboard.fd().unwrap();
    let fd = file.as_raw_fd();

    // Why do I have to do this...
    // When `file` dropped, its destructor gets called, and `fd` becomes invalid.
    // TODO: Fill a bug report to evdev-rs
    std::mem::forget(file);

    fd
  }
}

impl AsyncWorker for KeyPressWorker {
  fn step(&mut self, _: &mut WorkerManager) {
    let mut flag = evdev::ReadFlag::NORMAL;
    loop {
      match self.actual_keyboard.next_event(flag) {
        Ok((evdev::ReadStatus::Success, event)) => self.handle_event(event),
        Ok((evdev::ReadStatus::Sync, event)) => {
          warn!("Nasskan could not keep up with you typing so fast... now trying to recover.");
          flag = evdev::ReadFlag::SYNC;
          self.handle_event(event);
        }
        Err(nix::errno::Errno::EAGAIN) => return,
        Err(nix::errno::Errno::ENODEV) => return,
        Err(error) => {
          error!("evdev error: {:?}", error);
          return;
        }
      };
    }
  }
}

fn find_keyboards(ctx: &udev::Context) -> udev::Devices {
  let mut scanner = udev::Enumerator::new(ctx).unwrap();
  scanner.match_subsystem("input").unwrap();
  scanner.scan_devices().unwrap()
}

fn main() {
  match std::env::var("RUST_LOG") {
    Ok(_) => env_logger::init(),
    Err(_) => env_logger::builder()
      .filter_level(LevelFilter::Trace)
      .init(),
  }

  let mut manager = WorkerManager::new();
  let ctx = udev::Context::new().unwrap();
  let worker = KeyboardConnectionWorker::new(&ctx);
  // It's safe to use 0 as worker id because udev::Device::devnum never returns Some(0)
  manager.start(0, worker);
  info!("Start watching keyboard connections...");

  for keyboard in find_keyboards(&ctx) {
    let device_id = match keyboard.devnum() {
      Some(devnum) => devnum,
      None => continue,
    };

    if let Some(worker) = KeyPressWorker::for_keyboard(&keyboard) {
      info!("keyboard found!");
      manager.start(device_id.try_into().unwrap(), worker);
    }
  }

  manager.run()
}
