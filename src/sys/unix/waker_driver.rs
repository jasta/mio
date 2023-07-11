#[cfg(all(
    not(mio_unsupported_force_waker_pipe),
    any(target_os = "linux", target_os = "android")
))]
mod eventfd {
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::io::FromRawFd;

    /// WakerDriver backed by `eventfd`.
    ///
    /// `eventfd` is effectively an 64 bit counter. All writes must be of 8
    /// bytes (64 bits) and are converted (native endian) into an 64 bit
    /// unsigned integer and added to the count. Reads must also be 8 bytes and
    /// reset the count to 0, returning the count.
    #[derive(Debug)]
    pub struct WakerDriver {
        fd: File,
    }

    impl AsRawFd for WakerDriver {
        fn as_raw_fd(&self) -> RawFd {
            self.fd.as_raw_fd()
        }
    }

    impl WakerDriver {
        pub fn new() -> io::Result<WakerDriver> {
            let fd = syscall!(eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK))?;
            let file = unsafe { File::from_raw_fd(fd) };

            Ok(WakerDriver { fd: file })
        }

        pub fn wake(&self) -> io::Result<()> {
            let buf: [u8; 8] = 1u64.to_ne_bytes();
            match (&self.fd).write(&buf) {
                Ok(_) => Ok(()),
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    // Writing only blocks if the counter is going to overflow.
                    // So we'll reset the counter to 0 and wake it again.
                    self.reset()?;
                    self.wake()
                }
                Err(err) => Err(err),
            }
        }

        #[allow(dead_code)]
        pub(crate) fn ack(&self) -> io::Result<()> {
            self.reset()
        }

        /// Reset the eventfd object, only need to call this if `wake` fails.
        pub fn reset(&self) -> io::Result<()> {
            let mut buf: [u8; 8] = 0u64.to_ne_bytes();
            match (&self.fd).read(&mut buf) {
                Ok(_) => Ok(()),
                // If the `Waker` hasn't been awoken yet this will return a
                // `WouldBlock` error which we can safely ignore.
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => Ok(()),
                Err(err) => Err(err),
            }
        }
    }
}

#[cfg(all(
    not(mio_unsupported_force_waker_pipe),
    any(target_os = "linux", target_os = "android")
))]
pub use self::eventfd::WakerDriver;

cfg_unix_kevent_waker! {
mod kqueue {
    use crate::sys::Selector;
    use crate::Token;

    use std::io;

    /// WakerDriver backed by kqueue user space notifications (`EVFILT_USER`).
    #[derive(Debug)]
    pub struct WakerDriver {
        token: Token,
    }

    impl WakerDriver {
        pub fn new(kq: RawFd, token: Token) -> io::Result<WakerDriver> {
            // First attempt to accept user space notifications.
            let mut kevent = kevent!(
                0,
                libc::EVFILT_USER,
                libc::EV_ADD | libc::EV_CLEAR | libc::EV_RECEIPT,
                token.0
            );

            syscall!(kevent(kq, &kevent, 1, &mut kevent, 1, ptr::null())).and_then(|_| {
                if (kevent.flags & libc::EV_ERROR) != 0 && kevent.data != 0 {
                    Err(io::Error::from_raw_os_error(kevent.data as i32))
                } else {
                    Ok(())
                }
            })?;

            Ok(WakerDriver { token })
        }

        pub fn wake(&self) -> io::Result<()> {
            let mut kevent = kevent!(
                0,
                libc::EVFILT_USER,
                libc::EV_ADD | libc::EV_RECEIPT,
                token.0
            );
            kevent.fflags = libc::NOTE_TRIGGER;

            syscall!(kevent(self.kq, &kevent, 1, &mut kevent, 1, ptr::null())).and_then(|_| {
                if (kevent.flags & libc::EV_ERROR) != 0 && kevent.data != 0 {
                    Err(io::Error::from_raw_os_error(kevent.data as i32))
                } else {
                    Ok(())
                }
            })
        }
    }
}

pub use self::kqueue::WakerDriver;
}

#[cfg(any(
    mio_unsupported_force_waker_pipe,
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
))]
mod pipe {
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::io::FromRawFd;

    /// WakerDriver backed by a unix pipe.
    ///
    /// WakerDriver controls both the sending and receiving ends and empties the pipe
    /// if writing to it (waking) fails.
    #[derive(Debug)]
    pub struct WakerDriver {
        sender: File,
        receiver: File,
    }

    impl AsRawFd for WakerDriver {
        fn as_raw_fd(&self) -> RawFd {
            self.receiver.as_raw_fd()
        }
    }

    impl WakerDriver {
        pub fn new() -> io::Result<WakerDriver> {
            let mut fds = [-1; 2];
            syscall!(pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC))?;
            let sender = unsafe { File::from_raw_fd(fds[1]) };
            let receiver = unsafe { File::from_raw_fd(fds[0]) };

            Ok(WakerDriver { sender, receiver })
        }

        pub fn wake(&self) -> io::Result<()> {
            match (&self.sender).write(&[1]) {
                Ok(_) => Ok(()),
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    // The reading end is full so we'll empty the buffer and try
                    // again.
                    let _ = self.empty();
                    self.wake()
                }
                Err(ref err) if err.kind() == io::ErrorKind::Interrupted => self.wake(),
                Err(err) => Err(err),
            }
        }

        #[allow(dead_code)]
        pub(crate) fn ack(&self) -> io::Result<()> {
            self.empty()
        }

        /// Empty the pipe's buffer, only need to call this if `wake` fails.
        /// This ignores any errors.
        fn empty(&self) -> io::Result<()> {
            let mut buf = [0; 4096];
            loop {
                match (&self.receiver).read(&mut buf)? {
                    n if n > 0 => continue,
                    _ => return Ok(()),
                }
            }
        }
    }
}

#[cfg(any(
    mio_unsupported_force_waker_pipe,
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
))]
pub use self::pipe::WakerDriver;
