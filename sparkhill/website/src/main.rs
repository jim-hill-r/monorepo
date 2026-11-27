mod error_views;
mod templates;

use perseus::prelude::*;

// Export only: the site is built into static files (`perseus export`) and
// served by Cloudflare Pages, so it needs no server at runtime.
#[perseus::main_export]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new()
        .template(crate::templates::index::get_template())
        .error_views(crate::error_views::get_error_views())
        .locales_and_translations_manager("en-US", &[])
}
