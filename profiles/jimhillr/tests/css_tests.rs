use std::fs;
use std::path::Path;

#[test]
fn test_reset_css_exists() {
    let css_path = Path::new("static/css/reset.css");
    assert!(
        css_path.exists(),
        "reset.css should exist at static/css/reset.css"
    );
}

#[test]
fn test_reset_css_has_dark_mode_support() {
    let css_content = fs::read_to_string("static/css/reset.css")
        .expect("Failed to read reset.css");
    
    // Test that color-scheme property exists
    assert!(
        css_content.contains("color-scheme:"),
        "reset.css should contain color-scheme property"
    );
    
    // Test that it supports both light and dark modes
    assert!(
        css_content.contains("color-scheme: light dark") || 
        css_content.contains("color-scheme:light dark"),
        "reset.css should have 'color-scheme: light dark' to enable dark mode"
    );
}

#[test]
fn test_reset_css_applies_to_html_element() {
    let css_content = fs::read_to_string("static/css/reset.css")
        .expect("Failed to read reset.css");
    
    // Verify that color-scheme is set on the html element
    // This is important because color-scheme needs to be on the root element
    let lines: Vec<&str> = css_content.lines().collect();
    let mut in_html_block = false;
    let mut found_color_scheme = false;
    
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("html") {
            in_html_block = true;
        } else if in_html_block && trimmed.contains("color-scheme:") {
            found_color_scheme = true;
            break;
        } else if in_html_block && trimmed == "}" {
            break;
        }
    }
    
    assert!(
        found_color_scheme,
        "color-scheme property should be set within the html selector block"
    );
}

#[test]
fn test_reset_css_structure() {
    let css_content = fs::read_to_string("static/css/reset.css")
        .expect("Failed to read reset.css");
    
    // Verify basic CSS reset structure is intact
    assert!(css_content.contains("box-sizing: border-box"), "Should have box-sizing reset");
    assert!(css_content.contains("margin: 0"), "Should reset margins");
    assert!(css_content.contains("padding: 0"), "Should reset padding");
}
