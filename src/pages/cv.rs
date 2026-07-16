use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::{Container, SectionHeading, SkillBadge, TimelineEntry};
use crate::content::portfolio;
use crate::routes::{CV, title_for_path};

#[component]
pub fn CvPage() -> impl IntoView {
    let content = portfolio();
    let profile = content.profile;

    view! {
        <Title text=title_for_path(CV.path) />
        <Meta name="description" content="Professional summary, experience, education and skills." />

        <section class="page-hero page-hero--cv">
            <Container>
                <div class="cv-heading">
                    <div>
                        <p class="eyebrow">"Curriculum vitae / Sample content"</p>
                        <h1>{profile.name}</h1>
                        <p class="page-hero__lead">{profile.headline}</p>
                    </div>
                    {match profile.cv_download_url {
                        Some(url) => view! {
                            <a class="button button--primary" href=url download="">
                                "Download CV" <span aria-hidden="true">"↓"</span>
                            </a>
                        }.into_any(),
                        None => view! {
                            <span class="button button--disabled" aria-disabled="true" title="Add public/cv/cv.pdf and configure its URL in src/content.rs">
                                "CV PDF not supplied" <span aria-hidden="true">"—"</span>
                            </span>
                        }.into_any(),
                    }}
                </div>
                <div class="cv-contact-row">
                    <a href=format!("mailto:{}", profile.email)>{profile.email}</a>
                    <span>{profile.location}</span>
                    <span>{profile.availability}</span>
                </div>
            </Container>
        </section>

        <section class="section cv-section">
            <Container>
                <div class="cv-layout">
                    <aside>
                        <p class="eyebrow">"Summary"</p>
                        <p>{profile.summary}</p>
                        <p class="placeholder-note">"Placeholder: replace all CV text in src/content.rs before publishing."</p>
                    </aside>
                    <div>
                        <SectionHeading eyebrow="01 / Experience" title="Professional experience" />
                        <div class="timeline">{content.experience.iter().map(|item| view! { <TimelineEntry item=*item /> }).collect_view()}</div>
                    </div>
                </div>
            </Container>
        </section>

        <section class="section section--surface cv-section">
            <Container>
                <div class="cv-layout">
                    <SectionHeading eyebrow="02 / Education" title="Education" />
                    <div class="timeline">{content.education.iter().map(|item| view! { <TimelineEntry item=*item /> }).collect_view()}</div>
                </div>
            </Container>
        </section>

        <section class="section cv-section">
            <Container>
                <div class="cv-layout">
                    <SectionHeading eyebrow="03 / Skills" title="Technical skills" />
                    <div class="skill-groups skill-groups--cv">
                        {content.skills.iter().map(|group| view! {
                            <article class="skill-group">
                                <h3>{group.category}</h3>
                                <p>{group.summary}</p>
                                <div class="skill-list">{group.skills.iter().map(|skill| view! { <SkillBadge skill=*skill /> }).collect_view()}</div>
                            </article>
                        }).collect_view()}
                    </div>
                </div>
            </Container>
        </section>
    }
}
