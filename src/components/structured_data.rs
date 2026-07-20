//! Structured data generated from the public site identity.

use leptos::prelude::*;
use leptos_meta::Script;

use crate::content::portfolio;
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::{PRODUCTION_ORIGIN, SITE_DESCRIPTION, SITE_NAME};

/// Adds public JSON-LD after the client application has mounted.
///
/// It contains only public identity fields already published by the generated
/// CV; email and other contact details stay out.
#[component]
pub fn StructuredData() -> impl IntoView {
    let json = structured_data_json();

    view! {
        <Script type_="application/ld+json">{json}</Script>
    }
}

/// Builds the site's public JSON-LD graph.
pub fn structured_data_json() -> String {
    let same_as = GENERATED_CV
        .profile
        .social_links
        .iter()
        .map(|link| format!("\"{}\"", json_string(link.url.as_ref())))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        concat!(
            "{{\"@context\":\"https://schema.org\",\"@graph\":[",
            "{{\"@type\":\"Person\",\"name\":\"{}\",\"jobTitle\":\"{}\",\"sameAs\":[{}]}},",
            "{{\"@type\":\"WebSite\",\"name\":\"{}\",\"description\":\"{}\",\"url\":\"{}\"}}",
            "]}}"
        ),
        json_string(GENERATED_CV.profile.full_name.as_ref()),
        json_string(portfolio().profile.role),
        same_as,
        json_string(SITE_NAME),
        json_string(SITE_DESCRIPTION),
        json_string(PRODUCTION_ORIGIN.as_str()),
    )
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}
