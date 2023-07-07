use std::io;
use std::os::fd::RawFd;
use crate::{Interest, Token};
use crate::sys::Selector;

#[derive(Debug)]
pub(crate) struct WakerRegistrar;

impl WakerRegistrar {
  pub fn register(selector: &Selector, fd: RawFd, token: Token) -> io::Result<Self> {
    selector.register(fd, token, Interest::READABLE)?;
    Ok(Self)
  }

  pub fn prepare_to_wake(&self) -> io::Result<()> {
    // Nothing to do in the case that we are using an edge-triggered API
    Ok(())
  }
}