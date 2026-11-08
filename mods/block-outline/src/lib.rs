//! Boldy Mods: a recolourable outline around the targeted block or mob, and an arrow counter.
#[cfg(target_arch = "wasm32")]
mod arena;
pub mod counter;
pub mod cps;
pub mod fps;
pub mod crosshair;
#[cfg(target_arch = "wasm32")]
mod guest;
#[cfg(test)]
mod host_dump;
pub mod hudedit;
pub mod jumpreset;
pub mod outline;
pub mod panel;
pub mod zoom;
