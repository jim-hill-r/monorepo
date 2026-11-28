//! Markdown prose (see `cookbook_core::markdown`) rendered as elements, never as injected HTML.

use cookbook_core::markdown::{self, Block, Inline};
use dioxus::prelude::*;

/// Renders the inline spans of one Markdown block.
pub fn render_inlines(spans: &[Inline]) -> Element {
    rsx! {
        for span in spans.iter() {
            match span {
                Inline::Text(text) => rsx! { "{text}" },
                Inline::Emphasis(text) => rsx! { em { "{text}" } },
                Inline::Strong(text) => rsx! { strong { "{text}" } },
            }
        }
    }
}

/// Renders Markdown prose (see `cookbook_core::markdown`) as elements, never
/// as injected HTML.
#[component]
pub fn MarkdownBlocks(markdown: &'static str) -> Element {
    let blocks = markdown::parse(markdown);
    rsx! {
        for block in blocks.iter() {
            match block {
                Block::Paragraph(spans) => rsx! { p { {render_inlines(spans)} } },
                Block::Heading { level: 1, content } => rsx! { h1 { {render_inlines(content)} } },
                Block::Heading { level: 2, content } => rsx! { h2 { {render_inlines(content)} } },
                Block::Heading { level: 3, content } => rsx! { h3 { {render_inlines(content)} } },
                Block::Heading { level: 4, content } => rsx! { h4 { {render_inlines(content)} } },
                Block::Heading { level: 5, content } => rsx! { h5 { {render_inlines(content)} } },
                Block::Heading { content, .. } => rsx! { h6 { {render_inlines(content)} } },
            }
        }
    }
}
