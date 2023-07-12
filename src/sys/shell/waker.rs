use std::io;

#[derive(Debug)]
pub struct Waker {}

impl Waker {
    pub fn wake(&self) -> io::Result<()> {
        os_required!();
    }
}
