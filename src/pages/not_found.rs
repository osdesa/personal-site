use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::{ButtonLink, Container};
use crate::routes::{HOME, title_for_path};

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Title text=title_for_path("/not-found") />
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
