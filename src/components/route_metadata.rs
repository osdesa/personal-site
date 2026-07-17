//! Client-side hand-off from static fallback metadata to route metadata.

#[cfg(target_arch = "wasm32")]
use leptos::prelude::Effect;

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
