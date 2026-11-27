use dioxus::prelude::*;
use ui::FamilyTreePage;

/// The `/family-tree` route: the shared family tree page from `ui`, the same
/// one the web app shows.
#[component]
pub fn FamilyTree() -> Element {
    rsx! { FamilyTreePage {} }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_shared_family_tree_page() {
        let html = dioxus_ssr::render_element(rsx! { FamilyTree {} });
        assert!(html.contains("<h2>Family Tree</h2>"));
        assert!(html.contains("class=\"family-tree\""));
    }
}
