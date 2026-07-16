use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::{Container, ProjectCard};
use crate::content::portfolio;
use crate::routes::{PROJECTS, title_for_path};

#[component]
pub fn ProjectsPage() -> impl IntoView {
    let projects = portfolio().projects;

    view! {
        <Title text=title_for_path(PROJECTS.path) />
        <Meta name="description" content="Selected software engineering projects and technical case studies." />

        <section class="page-hero page-hero--compact">
            <Container>
                <p class="eyebrow">"Projects / Example entries"</p>
                <div class="page-hero__grid">
                    <h1>"Work explained with enough context to matter."</h1>
                    <div>
                        <p class="page-hero__lead">"Every card below is sample content that demonstrates the final information hierarchy."</p>
                        <p>"Real projects can include repositories, optional demonstrations, technologies and a featured state—all controlled from src/content.rs."</p>
                    </div>
                </div>
            </Container>
        </section>

        <section class="section">
            <Container>
                <div class="project-index-header">
                    <p>{format!("{} example projects", projects.len())}</p>
                    <p>"Replace before publishing"</p>
                </div>
                <div class="project-grid">
                    {projects.iter().enumerate().map(|(index, project)| view! {
                        <ProjectCard project=*project index=index />
                    }).collect_view()}
                </div>
            </Container>
        </section>
    }
}
