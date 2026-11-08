//! Page renderers. Pages never perform I/O; they read [`ShellState`] and emit
//! commands through `Context::listener`.

pub mod accounts;
pub mod downloads;
pub mod home;
pub mod instances;
pub mod settings;

use crate::app::SealyShell;
use crate::kit::{AnyElement, Context};
use crate::state::{Route, ShellState};

/// Renders the active route.
pub fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    match state.route {
        Route::Home => home::render(state, cx),
        Route::Instances => instances::render(state, cx),
        Route::Downloads => downloads::render(state, cx),
        Route::Accounts => accounts::render(state, cx),
        Route::Settings => settings::render(state, cx),
    }
}
