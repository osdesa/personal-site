use leptos::prelude::*;
use personal_site::pages::{LegalNoticePage, PrivacyNoticePage};
use personal_site::routes::{LEGAL_NOTICE, NAVIGATION_ROUTES, PRIVACY_NOTICE, metadata_for_path};

fn assert_has_non_empty_mailto(html: &str) {
    let target = html
        .split_once("href=\"mailto:")
        .and_then(|(_, remainder)| remainder.split_once('"'))
        .map(|(target, _)| target)
        .expect("notice must publish a mailto contact link");

    assert!(!target.trim().is_empty());
    assert!(target.contains('@'));
}

#[test]
fn legal_notice_renders_the_requested_terms_and_ownership_boundaries() {
    let html = view! { <LegalNoticePage /> }.to_html();

    for expected in [
        "Legal notice",
        "Website information",
        "Terms of use",
        "Intellectual property",
        "source code (where not licensed separately)",
        "GitHub's own terms and privacy policies",
        "Nothing on this website constitutes professional advice or creates any contractual or professional relationship.",
        "Disclaimer and liability",
        "Last updated",
    ] {
        assert!(
            html.contains(expected),
            "missing legal notice content: {expected}"
        );
    }
    assert_has_non_empty_mailto(&html);
}

#[test]
fn privacy_notice_renders_controller_hosting_and_rights_information() {
    let html = view! { <PrivacyNoticePage /> }.to_html();

    for expected in [
        "Privacy notice",
        "Data controller",
        "Contact details",
        "Email correspondence",
        "Cloudflare hosting",
        "Cloudflare processes technical information such as IP addresses, browser information and request metadata",
        "does not intentionally use advertising or behavioural tracking technologies",
        "UK GDPR information",
        "Your rights",
        "Information Commissioner's Office",
        "Last updated",
    ] {
        assert!(
            html.contains(expected),
            "missing privacy notice content: {expected}"
        );
    }
    assert_has_non_empty_mailto(&html);
}

#[test]
fn notices_are_public_but_not_primary_navigation() {
    for notice in [LEGAL_NOTICE, PRIVACY_NOTICE] {
        assert_eq!(metadata_for_path(notice.path), notice);
        assert!(
            NAVIGATION_ROUTES
                .iter()
                .all(|route| route.path != notice.path)
        );
    }
}

#[test]
fn legacy_legal_notice_path_has_a_permanent_host_redirect() {
    let redirects = include_str!("../public/_redirects");

    assert!(
        redirects
            .lines()
            .any(|line| line == "/legal-notice /legal 301")
    );
}

#[test]
fn notice_heroes_use_shared_heading_spacing() {
    let css = include_str!("../styles/input.css");

    assert!(css.contains(".page-hero--compact h1"));
    assert!(css.contains("margin-bottom: clamp(1.25rem, 2.5vw, 2rem)"));
}
