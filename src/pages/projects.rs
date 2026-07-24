use leptos::prelude::*;

use crate::components::{Container, ProjectCard, RouteMetadata};
use crate::generated_projects::PROJECTS as projects;
use crate::routes::PROJECTS;

#[component]
pub fn ProjectsPage() -> impl IntoView {
    view! {
        <RouteMetadata route=PROJECTS />

        <section class="section" aria-labelledby="selected-projects-title">
            <Container>
                <div class="project-index-header">
                    <h1 id="selected-projects-title" class="project-index-header__title">"Projects"</h1>
                    <p>"Newest first"</p>
                </div>
                <div class="project-grid">
                    {projects.iter().map(|project| view! {
                        <ProjectCard project=*project />
                    }).collect_view()}
                </div>
            </Container>
        </section>
    }
}
