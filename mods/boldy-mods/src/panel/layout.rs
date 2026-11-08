//! The static JSON-UI document: the card, its pages and their controls.

use super::help::TIP_WIDTH;
use super::*;

mod pages;
mod sections;
mod widgets;

pub(in crate::panel) use pages::*;
pub(in crate::panel) use sections::*;
pub(in crate::panel) use widgets::*;
