//! Implementation details for when we need to mimic an edge-triggered backend but actually have a
//! level-triggered backend (e.g. poll).

cfg_io_source! {
    pub(super) mod io_source_state;
}
