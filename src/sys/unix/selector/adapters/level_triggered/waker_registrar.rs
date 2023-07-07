use std::io;
use std::os::fd::RawFd;
use crate::sys::Selector;
use crate::{Interest, Token};

#[derive(Debug)]
pub(crate) struct WakerRegistrar {
  selector: Selector,
  fd: RawFd,
  token: Token,
}

impl WakerRegistrar {
  pub fn register(selector: &Selector, fd: RawFd, token: Token) -> io::Result<Self> {
    selector.register(fd, token, Interest::READABLE)?;
    Ok(WakerRegistrar {
      selector: selector.try_clone().unwrap(),
      fd,
      token,
    })
  }

  pub fn prepare_to_wake(&self) -> io::Result<()> {
    self.selector.reregister(self.fd, self.token, Interest::READABLE)
  }
}