use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::{ButtonLink, Container, remove_static_description_on_mount};
use crate::routes::{HOME, metadata_for_path};

#[component]
pub fn NotFoundPage() -> impl IntoView {
    let metadata = metadata_for_path("/not-found");
    remove_static_description_on_mount();

    view! {
        <Title text=metadata.title />
        <Meta name="description" content=metadata.description />
        <section class="not-found">
            <Container>
                <div class="not-found__code" aria-hidden="true">"404"</div>
                <p class="eyebrow">"Route not found"</p>
                <h1>"This page is outside the system."</h1>
                <p>"The address may have changed, or the page may not exist yet. The homepage is a reliable way back."</p>
                <ButtonLink href=HOME.path>"Return home"</ButtonLink>
            </Container>
        </section>
    }
}
