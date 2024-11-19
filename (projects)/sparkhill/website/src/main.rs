mod error_views;
mod templates;

use perseus::prelude::*;

#[perseus::main(perseus_axum::dflt_server)]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new()
        .template(crate::templates::index::get_template())
        .error_views(crate::error_views::get_error_views())
    // TODO: Understand how to use this injection to grab translations
    // .locales_and_translations_manager(
    //     "en-US", // Default locale
    //     &[],     // Other supported locales
    // )
}
