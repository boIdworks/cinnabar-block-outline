//! Host-independent block outline logic: settings, colours, the block raycast and box edges.
//! Kept free of Cinnabar bindings so `cargo test` exercises it natively.

mod colour;
mod fields;
mod raycast;
mod settings;

pub use colour::*;
pub use fields::*;
pub use raycast::*;
pub use settings::*;

#[cfg(test)]
mod tests;
