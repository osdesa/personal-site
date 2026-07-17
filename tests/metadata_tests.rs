use std::fs;

use personal_site::components::structured_data_json;
use personal_site::routes::{HOME, SITE_DESCRIPTION, SITE_NAME};

#[test]
fn initial_document_has_truthful_site_wide_share_metadata() {
    let document = include_str!("../index.html");

    for expected in [
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
fn initial_document_does_not_claim_origin_dependent_metadata() {
    let document = include_str!("../index.html");

    for absent in [
        "rel=\"canonical\"",
        "property=\"og:url\"",
        "property=\"og:image\"",
    ] {
        assert!(
            !document.contains(absent),
            "origin-dependent metadata must wait for a canonical public origin: {absent}"
        );
    }
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
    assert!(!json.contains("\"url\""));
}
