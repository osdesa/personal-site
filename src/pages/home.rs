use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::{ButtonLink, Container, ProjectCard};
use crate::content::portfolio;
use crate::cv_presentation::RichTextView;
use crate::generated_cv::CV as GENERATED_CV;
use crate::generated_projects::PROJECTS as GENERATED_PROJECTS;
use crate::routes::{CV, HOME, PROJECTS, title_for_path};

#[component]
pub fn HomePage() -> impl IntoView {
    let content = portfolio();
    let profile = content.profile;
    let imported_profile = &GENERATED_CV.profile;

    view! {
        <Title text=title_for_path(HOME.path) />
        <Meta
            name="description"
            content="Hayden Farrell - software engineer and computer science student."
        />

        <section class="home-page">
            <div class="container home-page__inner">
                <div class="home-page__intro">
                    <h1>{imported_profile.full_name.as_ref()}</h1>
                    <p class="home-page__role">{profile.role}</p>
                    <p class="home-page__summary">{profile.home_intro}</p>

                    <div class="home-page__actions">
                        <ButtonLink href=PROJECTS.path>"View projects"</ButtonLink>
                        <ButtonLink href=CV.path secondary=true>"View CV"</ButtonLink>
                    </div>
                </div>

                <ul class="home-socials" aria-label="Professional profiles">
                    {imported_profile.social_links.iter().map(|link| view! {
                        <li>
                            <a href=link.url.as_ref() target="_blank" rel="noreferrer">
                                <RichTextView text=&link.label />
                                <span class="sr-only">" (opens in a new tab)"</span>
                            </a>
                        </li>
                    }).collect_view()}
                    <li>
                        <a href=format!("mailto:{}", imported_profile.contact.email)>"Email"</a>
                    </li>
                </ul>
            </div>
        </section>

        <section class="section home-projects" aria-labelledby="selected-projects-title">
            <Container>
                <div class="section-heading-row">
                    <div class="section-heading">
                        <p class="eyebrow">"Selected projects"</p>
                        <h2 id="selected-projects-title">"Recent work"</h2>
                    </div>
                    <ButtonLink href=PROJECTS.path secondary=true>"View all projects"</ButtonLink>
                </div>
                <div class="project-grid">
                    {GENERATED_PROJECTS.iter().enumerate().map(|(index, project)| view! {
                        <ProjectCard project=*project index=index />
                    }).collect_view()}
                </div>
            </Container>
        </section>
    }
}
