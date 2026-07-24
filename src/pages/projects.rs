use leptos::prelude::*;

use crate::components::{
    Container, ProjectCard, RouteMetadata, remove_static_description_on_mount,
};
use crate::generated_projects::PROJECTS as projects;
use crate::routes::{PROJECTS, metadata_for_path};

#[component]
pub fn ProjectsPage() -> impl IntoView {
    let metadata = metadata_for_path(PROJECTS.path);
    remove_static_description_on_mount();

    view! {
        <RouteMetadata route=metadata />

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
