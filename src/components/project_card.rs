use leptos::prelude::*;

use crate::components::SkillBadge;
use crate::projects::{Project, ProjectVisibility};

#[component]
pub fn ProjectCard(project: Project) -> impl IntoView {
    let visibility = match project.visibility {
        ProjectVisibility::Public => "Public",
        ProjectVisibility::Private => "Private repository",
    };
    let show_private_indicator =
        project.visibility == ProjectVisibility::Private && project.repository_url.is_none();
    let has_links = project.repository_url.is_some() || project.demo_url.is_some();

    view! {
        <article class="project-card">
            <div class="project-card__visual" aria-hidden="true">
                <img
                    class="project-card__image"
                    src=project.image_url
                    alt=""
                    width="608"
                    height="272"
                    loading="lazy"
                    decoding="async"
                />
                <span class="project-card__grid"></span>
            </div>
            <div class="project-card__body">
                <div class="project-card__meta">
                    <span>{visibility}</span>
                    <span aria-hidden="true">"•"</span>
                    <time datetime=project.project_date>{project.project_date}</time>
                    {project.status.map(|status| view! {
                        <span aria-hidden="true">"•"</span>
                        <span class=status_class(status)>{status}</span>
                    })}
                </div>
                <h2>{project.title}</h2>
                <p>{project.summary}</p>
                <div class="skill-list" aria-label="Technologies used">
                    {project.technologies.iter().map(|skill| view! { <SkillBadge skill=*skill /> }).collect_view()}
                </div>
                {(!project.highlights.is_empty()).then(|| view! {
                    <ul class="project-card__highlights" aria-label="Project highlights">
                        {project.highlights.iter().map(|highlight| view! { <li>{*highlight}</li> }).collect_view()}
                    </ul>
                })}
                {(has_links || show_private_indicator).then(|| view! {
                    <div class="project-card__links">
                        {project.repository_url.map(|url| view! {
                            <a href=url target="_blank" rel="noreferrer">
                                "Repository"
                                <span class="sr-only">{format!(" for {} (opens in a new tab)", project.title)}</span>
                                <span aria-hidden="true">"↗"</span>
                            </a>
                        })}
                        {show_private_indicator.then(|| view! {
                            <span class="project-card__private">"Private repository"</span>
                        })}
                        {project.demo_url.map(|url| view! {
                            <a href=url target="_blank" rel="noreferrer">
                                "Live demo"
                                <span class="sr-only">{format!(" for {} (opens in a new tab)", project.title)}</span>
                                <span aria-hidden="true">"↗"</span>
                            </a>
                        })}
                    </div>
                })}
            </div>
        </article>
    }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "Active" => "project-card__status project-card__status--active",
        "Completed" => "project-card__status project-card__status--completed",
        _ => "project-card__status",
    }
}
