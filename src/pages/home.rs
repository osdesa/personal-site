use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::ButtonLink;
use crate::content::portfolio;
use crate::routes::{CV, HOME, PROJECTS, title_for_path};

#[component]
pub fn HomePage() -> impl IntoView {
    let content = portfolio();
    let profile = content.profile;

    view! {
        <Title text=title_for_path(HOME.path) />
        <Meta
            name="description"
            content="Hayden Farrell - software engineer and computer science student."
        />

        <section class="home-page">
            <div class="container home-page__inner">
                <div class="home-page__intro">
                    <h1>{profile.name}</h1>
                    <p class="home-page__role">{profile.role}</p>
                    <p class="home-page__summary">{profile.home_intro}</p>

                    <div class="home-page__actions">
                        <ButtonLink href=PROJECTS.path>"View projects"</ButtonLink>
                        <ButtonLink href=CV.path secondary=true>"View CV"</ButtonLink>
                    </div>
                </div>

                <ul class="home-socials" aria-label="Professional profiles">
                    {content.social_links.iter().map(|link| {
                        let opens_new_tab = !link.url.starts_with("mailto:");
                        view! {
                            <li>
                                <a
                                    href=link.url
                                    target=opens_new_tab.then_some("_blank")
                                    rel=opens_new_tab.then_some("noreferrer")
                                >
                                    {link.label}
                                    <span class="sr-only">{if opens_new_tab { " (opens in a new tab)" } else { "" }}</span>
                                </a>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </div>
        </section>
    }
}
