cfg_epoll_selector! {
    mod edge_triggered;

    cfg_io_source! {
        pub(crate) use self::edge_triggered::io_source_state::IoSourceState;
    }
}

cfg_kqueue_selector! {
    mod edge_triggered;

    cfg_io_source! {
        pub(crate) use self::edge_triggered::io_source_state::IoSourceState;
    }
}

cfg_poll_selector! {
    mod level_triggered;

    cfg_io_source! {
        pub(crate) use self::level_triggered::io_source_state::IoSourceState;
    }
}

