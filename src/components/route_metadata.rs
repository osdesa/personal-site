//! Client-side hand-off from static fallback metadata to route metadata.

#[cfg(target_arch = "wasm32")]
use leptos::prelude::Effect;
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

use crate::routes::{RouteInfo, canonical_url_for_path};

/// Removes the static description after the route-level `Meta` component has
/// mounted, leaving one browser-visible description for the current route.
///
/// The static tag remains essential in the initial CSR document for crawlers
/// that do not execute Wasm. Native rendering intentionally does nothing.
pub fn remove_static_description_on_mount() {
    #[cfg(target_arch = "wasm32")]
    Effect::new(|_| {
        if let Some(static_description) = leptos::web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("site-description"))
        {
            static_description.remove();
        }
    });
}

/// Adds route-specific browser metadata after the CSR application mounts.
///
/// The initial document deliberately remains a generic site-wide fallback for
/// non-rendering crawlers. This component improves the browser and
/// JavaScript-capable crawler view without claiming that the static document is
/// route-specific.
#[component]
pub fn RouteMetadata(route: RouteInfo) -> impl IntoView {
    let canonical_url = canonical_url_for_path(route.path);

    view! {
        <Title text=route.title />
        <Meta name="description" content=route.description />
        <Link rel="canonical" href=canonical_url.clone() />
        <Meta property="og:url" content=canonical_url />
    }
}
