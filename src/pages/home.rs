use leptos::prelude::*;

use crate::components::{ButtonLink, RouteMetadata};
use crate::content::portfolio;
use crate::cv::SocialPlatform;
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::{CV, HOME, PROJECTS};

#[component]
pub fn HomePage() -> impl IntoView {
    let content = portfolio();
    let profile = content.profile;
    let imported_profile = &GENERATED_CV.profile;
    view! {
        <RouteMetadata route=HOME />

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
                    <li>
                        <a href=format!("mailto:{}", imported_profile.contact.email)>"Email"</a>
                    </li>
                    {imported_profile.social_links.iter().filter(|link| link.platform == SocialPlatform::LinkedIn).map(|link| view! {
                        <li>
                            <a href=link.url.as_ref() target="_blank" rel="noreferrer">
                                "LinkedIn"
                                <span class="sr-only">" (opens in a new tab)"</span>
                            </a>
                        </li>
                    }).collect_view()}
                    {imported_profile.social_links.iter().filter(|link| link.platform == SocialPlatform::GitHub).map(|link| view! {
                        <li>
                            <a href=link.url.as_ref() target="_blank" rel="noreferrer">
                                "GitHub"
                                <span class="sr-only">" (opens in a new tab)"</span>
                            </a>
                        </li>
                    }).collect_view()}
                </ul>
            </div>
        </section>
    }
}
