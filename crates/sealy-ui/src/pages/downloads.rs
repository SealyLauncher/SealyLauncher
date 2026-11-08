//! Version browser.

use crate::app::SealyShell;
use crate::kit::{AnyElement, Context};
use crate::pages::placeholder;
use crate::state::ShellState;

/// Renders the downloads page.
pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder::render_downloads(state, cx)
}
