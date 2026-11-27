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
        // Browsers ask for /favicon.ico on pages with no icon link, such as
        // the locale redirect at /, so the export puts a copy there too.
        .static_alias("/favicon.ico", "static/favicon.ico")
        .locales_and_translations_manager("en-US", &[])
}
