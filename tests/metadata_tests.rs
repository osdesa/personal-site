use std::fs;

use personal_site::components::structured_data_json;
use personal_site::routes::{
    HOME, PRODUCTION_ORIGIN, PUBLIC_ROUTES, SITE_DESCRIPTION, SITE_NAME, canonical_url_for_path,
    social_image_url,
};

#[test]
fn initial_document_has_truthful_site_wide_share_metadata() {
    let document = include_str!("../index.html");

    for expected in [
        "id=\"site-canonical\"",
        "id=\"site-og-url\"",
        "<meta property=\"og:type\" content=\"website\" />",
        "<meta property=\"og:site_name\" content=\"Hayden Farrell\" />",
        "<meta name=\"twitter:card\" content=\"summary\" />",
        "<meta name=\"color-scheme\" content=\"dark\" />",
    ] {
        assert!(
            document.contains(expected),
            "missing static metadata: {expected}"
        );
    }

    assert!(document.contains(SITE_NAME));
    assert!(document.contains(SITE_DESCRIPTION));
    assert!(document.contains(HOME.title));
}

#[test]
fn static_metadata_has_stable_client_handoff_identifiers() {
    let document = include_str!("../index.html");

    for id in ["site-description", "site-canonical", "site-og-url"] {
        assert_eq!(
            document.matches(&format!("id=\"{id}\"")).count(),
            1,
            "static metadata id must be unique: {id}"
        );
    }
}

#[test]
fn static_document_has_canonical_production_metadata() {
    let document = include_str!("../index.html");

    for expected in [
        format!(
            "rel=\"canonical\" href=\"{}\"",
            canonical_url_for_path(HOME.path)
        ),
        format!(
            "property=\"og:url\" content=\"{}\"",
            canonical_url_for_path(HOME.path)
        ),
        format!("property=\"og:image\" content=\"{}\"", social_image_url()),
        format!("name=\"twitter:image\" content=\"{}\"", social_image_url()),
    ] {
        assert!(
            document.contains(&expected),
            "missing static production metadata: {expected}"
        );
    }
}

#[test]
fn canonical_urls_are_derived_from_the_one_typed_origin() {
    assert_eq!(PRODUCTION_ORIGIN.as_str(), "https://haydenfarrell.dev");
    for route in PUBLIC_ROUTES {
        assert_eq!(
            canonical_url_for_path(route.path),
            format!("https://haydenfarrell.dev{}", route.path)
        );
    }
    assert_eq!(
        social_image_url(),
        "https://haydenfarrell.dev/images/project-default.svg"
    );
}

#[test]
fn crawl_control_files_contain_only_the_public_production_routes() {
    let robots = include_str!("../public/robots.txt");
    let sitemap = include_str!("../public/sitemap.xml");

    assert!(robots.contains("User-agent: *"));
    assert!(robots.contains("Allow: /"));
    assert!(robots.contains("Sitemap: https://haydenfarrell.dev/sitemap.xml"));
    for path in ["/", "/projects", "/cv", "/legal", "/privacy"] {
        assert!(sitemap.contains(&canonical_url_for_path(path)));
    }
    assert_eq!(sitemap.matches("<loc>").count(), 5);
    assert!(!sitemap.contains("/legal-notice"));
    assert!(!sitemap.contains("not-found"));
}

#[test]
fn initial_document_references_a_controlled_favicon() {
    let document = include_str!("../index.html");

    assert!(document.contains("href=\"/favicon.svg\""));
    assert!(fs::metadata("public/favicon.svg").is_ok());
}

#[test]
fn structured_data_uses_public_identity_without_contact_details() {
    let json = structured_data_json();

    assert!(json.contains("\"@type\":\"Person\""));
    assert!(json.contains("\"@type\":\"WebSite\""));
    assert!(json.contains("\"sameAs\""));
    assert!(!json.contains("haydenfarrell@outlook.com"));
    assert!(json.contains("\"url\":\"https://haydenfarrell.dev\""));
}
