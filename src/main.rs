use zellij_tile::prelude::*;

// Register the main UI state plugin from your library crate
register_plugin!(zellij_layoutswitch::State);

// Register the background worker state from your library crate
register_worker!(
    zellij_layoutswitch::LayoutWorker, 
    layout_worker, 
    LAYOUT_WORKER
);

