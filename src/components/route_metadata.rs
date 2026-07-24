//! Client-side hand-off from static fallback metadata to route metadata.

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

use crate::routes::{RouteInfo, canonical_url_for_path};

/// Adds route-specific browser metadata after the CSR application mounts.
///
/// The initial document deliberately remains a generic site-wide fallback for
/// non-rendering crawlers. This component improves the browser and
/// JavaScript-capable crawler view without claiming that the static document is
/// route-specific.
#[component]
pub fn RouteMetadata(route: RouteInfo) -> impl IntoView {
    let canonical_url = canonical_url_for_path(route.path);
    remove_static_fallback_metadata_on_mount();

    view! {
        <Title text=route.title />
        <Meta name="description" content=route.description />
        <Link id="route-canonical" rel="canonical" href=canonical_url.clone() />
        <Meta property="og:url" content=canonical_url />
        {route.robots.map(|content| view! { <Meta name="robots" content /> })}
    }
}

/// Removes generic crawler fallbacks once their route-specific replacements
/// have mounted. Native rendering intentionally leaves the static document
/// boundary alone.
fn remove_static_fallback_metadata_on_mount() {
    #[cfg(target_arch = "wasm32")]
    Effect::new(|_| {
        let Some(document) = leptos::web_sys::window().and_then(|window| window.document()) else {
            return;
        };

        for id in ["site-description", "site-canonical", "site-og-url"] {
            if let Some(element) = document.get_element_by_id(id) {
                element.remove();
            }
        }
    });
}
