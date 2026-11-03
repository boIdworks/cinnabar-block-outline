//! Block outline: a recolourable outline around the targeted block.
#[cfg(target_arch = "wasm32")]
mod arena;
#[cfg(target_arch = "wasm32")]
mod guest;
pub mod outline;
pub mod panel;
