use dioxus::prelude::*;

const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");

#[component]
pub fn Sidebar(is_open: Signal<bool>, children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }

        div {
            id: "sidebar",
            class: if is_open() { "open" } else { "" },
            div {
                class: "sidebar-content",
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_css_asset_path() {
        // Verify the CSS asset path is correctly defined
        let asset_path = SIDEBAR_CSS.to_string();
        assert!(asset_path.contains("sidebar.css"));
    }

    #[test]
    fn test_sidebar_component_module_exists() {
        // Verify the Sidebar component is exported and accessible
        // This ensures the module compiles and the component is public
        use crate::Sidebar;
        let _ = Sidebar;
    }
}
