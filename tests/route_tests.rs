use std::collections::HashSet;

use personal_site::routes::{NAVIGATION_ROUTES, metadata_for_path, title_for_path};

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
    for route in NAVIGATION_ROUTES {
        assert!(title_for_path(route.path).contains("Hayden Farrell"));
    }
    assert!(title_for_path("/missing").starts_with("Page not found"));
}

#[test]
fn every_route_has_a_unique_non_empty_description() {
    let descriptions: HashSet<_> = NAVIGATION_ROUTES
        .iter()
        .map(|route| route.description)
        .collect();

    assert_eq!(descriptions.len(), NAVIGATION_ROUTES.len());
    assert!(
        NAVIGATION_ROUTES
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
    assert!(title_for_path("/about").starts_with("Page not found"));
    assert!(title_for_path("/contact").starts_with("Page not found"));
}
