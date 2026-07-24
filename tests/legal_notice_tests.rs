use leptos::prelude::*;
use personal_site::generated_cv::CV;
use personal_site::pages::LegalNoticePage;
use personal_site::routes::{LEGAL_NOTICE, NAVIGATION_ROUTES, metadata_for_path};

#[test]
fn legal_notice_renders_operator_privacy_and_contact_information() {
    let html = view! { <LegalNoticePage /> }.to_html();

    for expected in [
        "Legal notice",
        "Site operator",
        "Privacy and data handling",
        "Cloudflare Pages",
        "Information Commissioner's Office",
        &format!("href=\"mailto:{}\"", CV.profile.contact.email),
    ] {
        assert!(
            html.contains(expected),
            "missing legal notice content: {expected}"
        );
    }
}

#[test]
fn legal_notice_is_public_but_not_primary_navigation() {
    assert_eq!(metadata_for_path(LEGAL_NOTICE.path), LEGAL_NOTICE);
    assert!(
        NAVIGATION_ROUTES
            .iter()
            .all(|route| route.path != LEGAL_NOTICE.path)
    );
}
