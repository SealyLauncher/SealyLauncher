//! Accounts.

use crate::app::SealyShell;
use crate::kit::{AnyElement, Context};
use crate::pages::placeholder;
use crate::state::ShellState;

/// Renders the accounts page.
pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder::render_accounts(state, cx)
}
