//! Boldy Mods: a recolourable outline around the targeted block or mob, and an arrow counter.
#[cfg(target_arch = "wasm32")]
mod arena;
#[cfg(target_arch = "wasm32")]
mod guest;
pub mod counter;
pub mod outline;
pub mod panel;
pub mod zoom;
