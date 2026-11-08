//! Asset source for the launcher: Lucide icons from the kit plus our own art.

use std::borrow::Cow;

use gpui_kit::AssetSource;

/// The launcher icon, embedded so a portable copy never misses it.
const ICON: &[u8] = include_bytes!("../assets/icon.png");
/// Asset path used by [`crate::app`] when drawing the icon.
pub const ICON_PATH: &str = "sealy/icon.png";

/// Serves the kit's full icon set plus this crate's images.
pub struct SealyAssets;

impl AssetSource for SealyAssets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<Cow<'static, [u8]>>> {
        if path == ICON_PATH {
            return Ok(Some(Cow::Borrowed(ICON)));
        }
        gpui_kit::assets::AllAssets.load(path)
    }

    fn list(&self, path: &str) -> gpui_kit::Result<Vec<gpui_kit::SharedString>> {
        let mut paths = gpui_kit::assets::AllAssets.list(path)?;
        if path.is_empty() || ICON_PATH.starts_with(path) {
            paths.push(ICON_PATH.into());
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
