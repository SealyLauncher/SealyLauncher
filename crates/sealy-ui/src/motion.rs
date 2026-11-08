//! Motion tokens shared by Phase 0 and later pages.

use std::time::Duration;

/// Page transition duration.
pub const PAGE: Duration = Duration::from_millis(240);

/// Cubic ease-out used by the Phase 0 page transition.
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
