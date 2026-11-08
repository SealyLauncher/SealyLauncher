//! Instance list.

use crate::app::SealyShell;
use crate::kit::{AnyElement, Context};
use crate::pages::placeholder;
use crate::state::ShellState;

/// Renders the instance page.
pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder::render_instances(state, cx)
}
