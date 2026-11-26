use cahokia_core::sample::sample_family_tree;
use dioxus::prelude::*;
use ui::FamilyTreeView;

/// The family tree page. Until people can enter their own family it shows
/// the made-up sample family from `cahokia_core`.
#[component]
pub fn FamilyTree() -> Element {
    rsx! {
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
        dioxus_ssr::render_element(rsx! { FamilyTree {} })
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
