//! Boldy Mods: a recolourable outline around the targeted block or mob, and an arrow counter.
#[cfg(target_arch = "wasm32")]
mod arena;
pub mod counter;
pub mod cps;
#[cfg(target_arch = "wasm32")]
mod guest;
pub mod hudedit;
pub mod outline;
pub mod panel;
pub mod zoom;
