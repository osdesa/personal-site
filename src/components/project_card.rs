use leptos::prelude::*;

use crate::components::SkillBadge;
use crate::content::Project;

#[component]
pub fn ProjectCard(project: Project, #[prop(optional)] index: usize) -> impl IntoView {
    let number = format!("{:02}", index + 1);
    let has_demo = project.demo_url.is_some();

    view! {
        <article class="project-card">
            <div class="project-card__visual" aria-hidden="true">
                <span class="project-card__number">{number}</span>
                {project.image_url.map(|url| view! {
                    <img class="project-card__image" src=url alt="" loading="lazy" />
                })}
                <span class="project-card__grid"></span>
                {project.image_url.is_none().then(|| view! {
                    <span class="project-card__mark">{project.title.chars().find(char::is_ascii_alphabetic).unwrap_or('P')}</span>
                })}
            </div>
            <div class="project-card__body">
                <div class="project-card__meta">
                    <span>{if project.featured { "Featured sample" } else { "Sample project" }}</span>
                    <span aria-hidden="true">"•"</span>
                    <span>{project.technologies[0]}</span>
                </div>
                <h3>{project.title}</h3>
                <p>{project.description}</p>
                <div class="skill-list" aria-label="Technologies used">
                    {project.technologies.iter().map(|skill| view! { <SkillBadge skill=*skill /> }).collect_view()}
                </div>
                <div class="project-card__links">
                    <a href=project.repository_url target="_blank" rel="noreferrer">
                        "Repository"
                        <span class="sr-only">{format!(" for {} (opens in a new tab)", project.title)}</span>
                        <span aria-hidden="true">"↗"</span>
                    </a>
                    {project.demo_url.map(|url| view! {
                        <a href=url target="_blank" rel="noreferrer">
                            "Live example"
                            <span class="sr-only">{format!(" for {} (opens in a new tab)", project.title)}</span>
                            <span aria-hidden="true">"↗"</span>
                        </a>
                    })}
                </div>
                <span class="sr-only">{if has_demo { "Demonstration link available" } else { "No demonstration link supplied" }}</span>
            </div>
        </article>
    }
}
