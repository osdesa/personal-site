use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::{Container, ProjectCard};
use crate::generated_projects::PROJECTS as projects;
use crate::routes::{PROJECTS, title_for_path};

#[component]
pub fn ProjectsPage() -> impl IntoView {
    view! {
        <Title text=title_for_path(PROJECTS.path) />
        <Meta name="description" content="Selected software engineering projects and technical case studies." />

        <section class="page-hero page-hero--compact">
            <Container>
                <p class="eyebrow">"Projects / Selected work"</p>
                <div class="page-hero__grid">
                    <h1>"Work explained with enough context to matter."</h1>
                    <div>
                        <p class="page-hero__lead">"Selected engineering work, synchronized from GitHub and presented with the context that matters."</p>
                        <p>"Repository metadata provides the baseline; project-specific summaries and highlights add detail where useful."</p>
                    </div>
                </div>
            </Container>
        </section>

        <section class="section">
            <Container>
                <div class="project-index-header">
                    <p>{format!("{} selected projects", projects.len())}</p>
                    <p>"Newest first"</p>
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
