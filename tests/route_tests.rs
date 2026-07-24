use std::collections::HashSet;

use personal_site::routes::{NAVIGATION_ROUTES, PUBLIC_ROUTES, metadata_for_path};

#[test]
fn navigation_routes_are_unique_and_absolute() {
    let paths: HashSet<_> = NAVIGATION_ROUTES.iter().map(|route| route.path).collect();

    assert_eq!(paths.len(), NAVIGATION_ROUTES.len());
    assert!(
        NAVIGATION_ROUTES
            .iter()
            .all(|route| route.path.starts_with('/'))
    );
    assert!(
        NAVIGATION_ROUTES
            .iter()
            .all(|route| !route.label.is_empty())
    );
}

#[test]
fn every_public_route_has_a_specific_title() {
    for route in PUBLIC_ROUTES {
        assert!(route.title.contains("Hayden Farrell"));
        assert_eq!(route.robots, None);
    }
    let missing = metadata_for_path("/missing");
    assert!(missing.title.starts_with("Page not found"));
    assert_eq!(missing.robots, Some("noindex, nofollow"));
}

#[test]
fn every_route_has_a_unique_non_empty_description() {
    let descriptions: HashSet<_> = PUBLIC_ROUTES
        .iter()
        .map(|route| route.description)
        .collect();

    assert_eq!(descriptions.len(), PUBLIC_ROUTES.len());
    assert!(
        PUBLIC_ROUTES
            .iter()
            .all(|route| !route.description.is_empty())
    );
    assert!(
        metadata_for_path("/missing")
            .description
            .contains("could not be found")
    );
}

#[test]
fn removed_sections_are_not_public_routes() {
    assert!(
        NAVIGATION_ROUTES
            .iter()
            .all(|route| !matches!(route.path, "/about" | "/contact"))
    );
    assert!(
        metadata_for_path("/about")
            .title
            .starts_with("Page not found")
    );
    assert!(
        metadata_for_path("/contact")
            .title
            .starts_with("Page not found")
    );
}

#[test]
fn not_found_decoration_has_a_bounded_non_obstructive_scale() {
    let css = include_str!("../styles/input.css");

    assert!(css.contains(".not-found__code"));
    assert!(css.contains("font-size: clamp(6.5rem, 18vw, 12rem)"));
    assert!(!css.contains("transform: translate(-50%, -50%)"));
}

#[test]
fn home_heading_has_responsive_separation_from_supporting_content() {
    let css = include_str!("../styles/input.css");

    assert!(css.contains("margin-bottom: clamp(1.5rem, 2.5vw, 2.25rem)"));
    assert!(css.contains("line-height: 0.88"));
}
