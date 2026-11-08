//! Motion tokens shared by the shell and pages.

use std::time::Duration;

/// Micro-interaction duration.
pub const FAST: Duration = Duration::from_millis(120);
/// Default UI transition.
pub const BASE: Duration = Duration::from_millis(200);
/// Page transition duration.
pub const PAGE: Duration = Duration::from_millis(240);
/// Larger surface transition.
pub const SLOW: Duration = Duration::from_millis(320);

/// Cubic ease-out used by the page transition.
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Quintic ease-out for slightly snappier surfaces.
pub fn ease_out_quint(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(5)
}

/// Symmetric cubic ease-in-out.
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}
