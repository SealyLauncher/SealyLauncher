//! Material Design 3 design layer.
//!
//! Phase 0 prototype: palette derivation + core widgets, used to validate the
//! look and motion inside GPUI before the full `sealy-ui` rewrite.

pub mod demo;
pub mod palette;
pub mod widgets;

pub use palette::{M3Scheme, mix, rgba_of};
