//! The family tree page that web, desktop and mobile all route to.

use cahokia_core::sample::sample_family_tree;
use dioxus::prelude::*;

use crate::FamilyTreeView;

const FAMILY_TREE_PAGE_CSS: Asset = asset!("/assets/styling/family_tree_page.css");

/// The family tree page: a heading, a short note and the tree in a
/// container that scrolls sideways when the tree is wider than the screen.
///
/// Until people can enter their own family it shows the made-up sample
/// family from `cahokia_core`. Each platform app routes `/family-tree` to
/// this component, so they all show the same page.
#[component]
pub fn FamilyTreePage() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: FAMILY_TREE_PAGE_CSS }

        div {
            class: "page-content",
            h2 { "Family Tree" }
            p { "A sample family, three generations deep. Scroll sideways if the tree is wider than your screen." }
            div {
                class: "family-tree-scroll",
                FamilyTreeView { tree: sample_family_tree() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render() -> String {
        dioxus_ssr::render_element(rsx! { FamilyTreePage {} })
    }

    #[test]
    fn shows_the_sample_tree_in_a_scroll_container() {
        let html = render();
        assert!(html.contains("<h2>Family Tree</h2>"));
        assert!(html.contains("class=\"family-tree-scroll\""));
        assert!(html.contains("class=\"family-tree\""));
    }

    #[test]
    fn draws_every_sample_person() {
        let html = render();
        let tree = sample_family_tree();
        assert_eq!(
            html.matches("class=\"family-tree-person").count(),
            tree.people.len()
        );
        for person in tree.people.values() {
            assert!(html.contains(&format!(">{}</text>", person.name)));
        }
    }
}
