//! Hive Auto CS: a Cinnabar packet mod that sets up a Bedwars Squads custom server on The Hive.
//!
//! `!autocs` (or `!autocs <map>`) creates the server, loads the preset, sets the map and, once
//! eight players are in, puts the party on RED and everyone else on BLUE (see [`autocs`]). The
//! recorder also logs every server form, chat line and command to Cinnabar's `client.log`, which
//! is how Hive's menus were learned and how a changed menu can be diagnosed.

pub mod autocs;
pub mod commands;
pub mod packets;
pub mod recorder;
pub mod wire;

#[cfg(target_arch = "wasm32")]
mod guest;

#[cfg(test)]
mod tests;
