//! Macros to ease conditional code based on enabled features.

// Depending on the features not all macros are used.
#![allow(unused_macros)]

/// The `os-poll` feature is enabled.
macro_rules! cfg_os_poll {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "os-poll")]
            #[cfg_attr(docsrs, doc(cfg(feature = "os-poll")))]
            $item
        )*
    }
}

/// The `os-poll` feature is disabled.
macro_rules! cfg_not_os_poll {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "os-poll"))]
            $item
        )*
    }
}

/// The `os-ext` feature is enabled.
macro_rules! cfg_os_ext {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "os-ext")]
            #[cfg_attr(docsrs, doc(cfg(feature = "os-ext")))]
            $item
        )*
    }
}

/// The `net` feature is enabled.
macro_rules! cfg_net {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "net")]
            #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
            $item
        )*
    }
}

/// One of the features enabled that needs `IoSource`. That is `net` or `os-ext`
/// on Unix (for `pipe`).
macro_rules! cfg_io_source {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "net", all(unix, feature = "os-ext")))]
            #[cfg_attr(docsrs, doc(cfg(any(feature = "net", all(unix, feature = "os-ext")))))]
            $item
        )*
    }
}

/// The `os-ext` feature is enabled, or one of the features that need `os-ext`.
macro_rules! cfg_any_os_ext {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "os-ext", feature = "net"))]
            #[cfg_attr(docsrs, doc(cfg(any(feature = "os-ext", feature = "net"))))]
            $item
        )*
    }
}

/// The current platform supports epoll.
macro_rules! cfg_epoll_selector {
    ($($item:item)*) => {
        $(
            #[cfg(all(
                not(mio_unsupported_force_poll_poll),
                any(
                    target_os = "android",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                )))]
            $item
        )*
    }
}

macro_rules! cfg_poll_selector {
    ($($item:item)*) => {
        $(
            #[cfg(mio_unsupported_force_poll_poll)]
            $item
        )*
    }
}

/// The current platform supports kqueue.
macro_rules! cfg_kqueue_selector {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "tvos",
                target_os = "watchos",
            ))]
            $item
        )*
    }
}

macro_rules! cfg_selector_has_fd {
    ($($item:item)*) => {
        $(
            #[cfg(all(
                unix,
                not(mio_unsupported_force_poll_poll),
            ))]
            $item
        )*
    }
}

macro_rules! trace {
    ($($t:tt)*) => {
        log!(trace, $($t)*)
    }
}

macro_rules! warn {
    ($($t:tt)*) => {
        log!(warn, $($t)*)
    }
}

macro_rules! error {
    ($($t:tt)*) => {
        log!(error, $($t)*)
    }
}

macro_rules! log {
    ($level: ident, $($t:tt)*) => {
        #[cfg(feature = "log")]
        { log::$level!($($t)*) }
        // Silence unused variables warnings.
        #[cfg(not(feature = "log"))]
        { if false { let _ = ( $($t)* ); } }
    }
}
