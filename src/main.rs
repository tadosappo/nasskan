use evdev_rs as evdev;
use log::*;
use mio::unix::EventedFd;
use mio::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;

mod config;
use config::*;

trait AsyncWorker: AsRawFd {
  fn step(&mut self, manager: &mut WorkerManager);
}

struct WorkerManager {
  poll: mio::Poll,
  workers: HashMap<usize, Rc<RefCell<dyn AsyncWorker>>>,
}

impl WorkerManager {
  fn new() -> Self {
    Self {
      poll: Poll::new().unwrap(),
      workers: HashMap::new(),
    }
  }

  fn run(&mut self) {
    let mut events = Events::with_capacity(128);
    loop {
      self.poll.poll(&mut events, None).unwrap();

      for event in events.iter() {
        let worker = self.workers.get_mut(&event.token().0).unwrap();
        Rc::clone(worker).borrow_mut().step(self)

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

struct KeyPressWorker {
  file: std::fs::File,
}

impl KeyPressWorker {
  fn new(path: &std::path::Path) -> Self {
    let file = std::fs::File::open(path).unwrap();

    Self { file }
  }
}

impl AsRawFd for KeyPressWorker {
  fn as_raw_fd(&self) -> RawFd {
    self.file.as_raw_fd()
  }
}

impl AsyncWorker for KeyPressWorker {
  fn step(&mut self, _: &mut WorkerManager) {
    debug!("pressed!")
  }
}

struct KeyboardConnectionWorker {
  monitor: udev::MonitorSocket,
}

impl KeyboardConnectionWorker {
  fn new() -> Self {
    let ctx = udev::Context::new().unwrap();
    let mut builder = udev::MonitorBuilder::new(&ctx).unwrap();
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
    let event = match self.monitor.next() {
      Some(event) => event,
      None => {
        warn!("We are no longer able to observe keyboard connections.");
        return;
      }
    };

    let connected_device = event.device();
    let device_id = match connected_device.devnum() {
      Some(devnum) => devnum,
      None => return,
    };
    let device_file_path = match connected_device.devnode() {
      Some(devnode) => devnode,
      None => return,
    };

    match event.event_type() {
      udev::EventType::Add => {
        for config_device in CONFIG.devices.iter() {
          let is_connected = config_device.if_.iter().all(|(name, value)| {
            connected_device
              .property_value(name)
              .and_then(|x| x.to_str())
              == Some(value)
          });
          if is_connected {
            info!(
              "A keyboard connected!: {:?} {:?}",
              device_file_path, config_device.if_
            );
            manager.start(
              device_id.try_into().unwrap(),
              KeyPressWorker::new(device_file_path),
            );
            return;
          }
        }
      }
      udev::EventType::Remove => {
        info!("A keyboard disconnected: {:?}", device_file_path);
        manager.stop(device_id.try_into().unwrap());
      }
      _ => {}
    }
  }
}

fn main() {
  match std::env::var("RUST_LOG") {
    Ok(_) => env_logger::init(),
    Err(_) => env_logger::builder()
      .filter_level(LevelFilter::Trace)
      .init(),
  }

  let mut manager = WorkerManager::new();

  let worker = KeyboardConnectionWorker::new();
  manager.start(0, worker);
  info!("Start watching keyboard connections...");
  // It's safe to use 0 as worker id because udev::Device::devnum never returns Some(0)

  manager.run()
}
