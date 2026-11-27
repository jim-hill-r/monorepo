mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::Navbar;

mod sidebar;
pub use sidebar::Sidebar;

mod family_tree;
pub use family_tree::FamilyTreeView;

mod family_tree_page;
pub use family_tree_page::FamilyTreePage;

/// The design system theme every Cahokia app uses until the user picks
/// another (`design_system_dioxus::Stylesheets`).
pub const DEFAULT_THEME: &str = "earthy";
