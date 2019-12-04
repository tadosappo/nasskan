use std::os::unix::io::AsRawFd;
use udev;

const DEVICE_UPDATE: mio::Token = mio::Token(0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
          let event = monitor.next().unwrap();
          println!("{}", event.event_type());

          for property in event.device().properties() {
            println!("{:?} == {:?}", property.name(), property.value());
          }
        }
        mio::Token(_) => unreachable!(),
      }
    }
  }
}
