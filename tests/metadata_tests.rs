use std::fs;

use personal_site::components::structured_data_json;
use personal_site::routes::{
    HOME, PRODUCTION_ORIGIN, PUBLIC_ROUTES, SITE_DESCRIPTION, SITE_NAME, SOCIAL_IMAGE_PATH,
    canonical_url_for_path, social_image_url,
};
use serde_json::Value;

#[test]
fn initial_document_has_truthful_site_wide_share_metadata() {
    let document = include_str!("../index.html");

    for expected in [
        "id=\"site-canonical\"".to_owned(),
        "id=\"site-og-url\"".to_owned(),
        "<meta property=\"og:type\" content=\"website\" />".to_owned(),
        format!("<meta property=\"og:site_name\" content=\"{SITE_NAME}\" />"),
        "<meta name=\"twitter:card\" content=\"summary\" />".to_owned(),
        "<meta name=\"color-scheme\" content=\"dark\" />".to_owned(),
    ] {
        assert!(
            document.contains(&expected),
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
    let origin = PRODUCTION_ORIGIN.as_str();
    assert!(origin.starts_with("https://"));
    assert!(!origin.ends_with('/'));
    for route in PUBLIC_ROUTES {
        assert_eq!(
            canonical_url_for_path(route.path),
            format!("{origin}{}", route.path)
        );
    }
    assert_eq!(social_image_url(), format!("{origin}{SOCIAL_IMAGE_PATH}"));
}

#[test]
fn crawl_control_files_contain_only_the_public_production_routes() {
    let robots = include_str!("../public/robots.txt");
    let sitemap = include_str!("../public/sitemap.xml");

    assert!(robots.contains("User-agent: *"));
    assert!(robots.contains("Allow: /"));
    assert!(robots.contains(&format!(
        "Sitemap: {}/sitemap.xml",
        PRODUCTION_ORIGIN.as_str()
    )));
    for route in PUBLIC_ROUTES {
        assert!(sitemap.contains(&canonical_url_for_path(route.path)));
    }
    assert_eq!(sitemap.matches("<loc>").count(), PUBLIC_ROUTES.len());
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
    let document: Value = serde_json::from_str(&structured_data_json()).unwrap();
    let graph = document["@graph"].as_array().unwrap();
    let person = graph
        .iter()
        .find(|entry| entry["@type"] == "Person")
        .unwrap();
    let website = graph
        .iter()
        .find(|entry| entry["@type"] == "WebSite")
        .unwrap();

    assert!(
        person["name"]
            .as_str()
            .is_some_and(|name| !name.trim().is_empty())
    );
    assert!(person.get("email").is_none());
    assert!(person["sameAs"].as_array().is_some_and(|links| {
        links
            .iter()
            .all(|link| link.as_str().is_some_and(|url| url.starts_with("https://")))
    }));
    assert_eq!(website["url"], PRODUCTION_ORIGIN.as_str());
}
