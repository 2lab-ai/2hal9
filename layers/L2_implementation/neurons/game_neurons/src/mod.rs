// Module declarations for Ultima Offline PAL Edition

pub mod game;

#[cfg(target_arch = "wasm32")]
pub mod lib;

#[cfg(not(target_arch = "wasm32"))]
pub mod main;