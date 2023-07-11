//! Implementation details for when we have an edge-triggered backend (i.e. epoll and kqueue).

cfg_io_source! {
    pub(super) mod io_source_state;
}
