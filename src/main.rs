use log::*;
use std::os::unix::io::AsRawFd;
use udev;

const DEVICE_UPDATE: mio::Token = mio::Token(0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  let ctx = udev::Context::new()?;
  let mut builder = udev::MonitorBuilder::new(&ctx)?;
  builder.match_subsystem("input")?;
  let mut monitor = builder.listen()?;

  let poll = mio::Poll::new()?;
  poll.register(
    &mio::unix::EventedFd(&monitor.as_raw_fd()),
    DEVICE_UPDATE,
    mio::Ready::readable(),
    mio::PollOpt::edge(),
  )?;

  let mut events = mio::Events::with_capacity(128);
  loop {
    poll.poll(&mut events, None).unwrap();

    for event in events.iter() {
      match event.token() {
        DEVICE_UPDATE => {
          let event = monitor.next();
          if event.is_none() {
            warn!("We are no longer able to observe keyboard connections.");
            continue;
          }
          let event = event.unwrap();

          match event.event_type() {
            udev::EventType::Add => {
              let device = event.device();
              let vendor_id = device.property_value("ID_VENDOR").and_then(|x| x.to_str());
              let model_id = device.property_value("ID_MODEL").and_then(|x| x.to_str());

              if vendor_id == Some("05f3") && model_id == Some("0007") {
                debug!("Kinesis connected")
              }
            }
            _ => {}
          }
        }
        mio::Token(_) => unreachable!(),
      }
    }
  }
}
