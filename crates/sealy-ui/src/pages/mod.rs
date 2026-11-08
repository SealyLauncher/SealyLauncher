//! Page routing.

mod accounts;
mod downloads;
mod home;
mod instances;
mod oobe;
mod settings;

use crate::app::SealyShell;
use crate::kit::{AnyElement, Context};
use crate::state::{Route, ShellState};

/// Renders the active page below the window chrome.
pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    if let Some(step) = state.oobe_step {
        return oobe::render(state, step, cx);
    }

    match state.route {
        Route::Home => home::render(state, cx),
        Route::Instances => instances::render(state, cx),
        Route::Downloads => downloads::render(state, cx),
        Route::Accounts => accounts::render(state, cx),
        Route::Settings => settings::render(state, cx),
    }
}
