//! Leptos presentation for the generated curriculum-vitae domain model.
//!
//! This module consumes [`crate::cv`] values directly. It does not reinterpret
//! LaTeX, duplicate imported content, or render raw HTML.

use leptos::prelude::*;

use crate::components::{Container, SectionHeading};
use crate::cv::{
    Cv, CvDate, DateRange, DateRangeEnd, Education, Experience, Inline, Location, Month, RichText,
    SkillGroup, SocialLink, SocialPlatform,
};

/// Public URL copied by Trunk from the synchronized CV artifact directory.
pub const CV_PDF_URL: &str = "/cv/Hayden-Farrell-CV.pdf";
/// Suggested filename supplied to browsers for the synchronized PDF.
pub const CV_PDF_FILENAME: &str = "Hayden-Farrell-CV.pdf";

/// Renders the complete imported CV or a clear unavailable state.
///
/// The optional inputs make failure behaviour explicit and independently
/// testable. Production composition supplies the generated static value and
/// synchronized PDF URL.
#[component]
pub fn CvDocument(
    cv: Option<&'static Cv<'static>>,
    source_tag: &'static str,
    source_commit_sha: &'static str,
    pdf_url: Option<&'static str>,
) -> impl IntoView {
    match cv {
        Some(cv) => view! {
            <CvProfile cv=cv pdf_url=pdf_url />
            {(!cv.experience.is_empty()).then(|| view! { <ExperienceSection entries=&cv.experience /> })}
            {(!cv.education.is_empty()).then(|| view! { <EducationSection entries=&cv.education /> })}
            {(!cv.skills.is_empty()).then(|| view! { <SkillsSection groups=&cv.skills /> })}
            <CvSourceVersion tag=source_tag commit_sha=source_commit_sha />
        }
        .into_any(),
        None => view! { <CvUnavailable pdf_url=pdf_url /> }.into_any(),
    }
}

#[component]
fn CvProfile(cv: &'static Cv<'static>, pdf_url: Option<&'static str>) -> impl IntoView {
    let profile = &cv.profile;
    let email_url = format!("mailto:{}", profile.contact.email);

    view! {
        <section class="page-hero page-hero--cv" aria-labelledby="cv-title">
            <Container>
                <div class="cv-heading">
                    <div>
                        <p class="eyebrow">"Curriculum vitae"</p>
                        <h1 id="cv-title">{profile.full_name.as_ref()}</h1>
                        <p class="page-hero__lead">
                            "Professional experience, education and technical capabilities."
                        </p>
                    </div>
                    <PdfDownload pdf_url=pdf_url />
                </div>

                <address class="cv-contact-row">
                    <a href=email_url aria-label=format!("Email {}", profile.contact.email)>
                        {profile.contact.email.as_ref()}
                    </a>
                    {(!profile.social_links.is_empty()).then(|| view! {
                        <ul class="cv-profile-links" aria-label="Professional profiles">
                            {profile.social_links.iter().map(|link| view! {
                                <li><SocialProfileLink link=link /></li>
                            }).collect_view()}
                        </ul>
                    })}
                </address>
            </Container>
        </section>
    }
}

#[component]
fn PdfDownload(pdf_url: Option<&'static str>) -> impl IntoView {
    match pdf_url {
        Some(url) => view! {
            <a
                class="button button--primary cv-download"
                href=url
                download=CV_PDF_FILENAME
                aria-label="Download CV as a PDF"
            >
                "Download PDF"
                <span aria-hidden="true" class="button__arrow">"↓"</span>
            </a>
        }
        .into_any(),
        None => view! {
            <p class="cv-download-unavailable" role="status">
                <span class="button button--disabled" aria-disabled="true">"PDF unavailable"</span>
                <span>"The web CV is still available below."</span>
            </p>
        }
        .into_any(),
    }
}

#[component]
fn SocialProfileLink(link: &'static SocialLink<'static>) -> impl IntoView {
    let platform = platform_label(link.platform);

    view! {
        <a
            href=link.url.as_ref()
            target="_blank"
            rel="noreferrer"
            aria-label=format!("{platform} professional profile (opens in a new tab)")
        >
            <RichTextView text=&link.label />
            <span class="sr-only">" (opens in a new tab)"</span>
        </a>
    }
}

#[component]
fn ExperienceSection(entries: &'static [Experience<'static>]) -> impl IntoView {
    view! {
        <section class="section cv-section">
            <Container>
                <div class="cv-layout">
                    <SectionHeading eyebrow="Career" title="Professional experience" />
                    <div class="timeline">
                        {entries.iter().map(|entry| view! { <ExperienceEntry entry=entry /> }).collect_view()}
                    </div>
                </div>
            </Container>
        </section>
    }
}

#[component]
fn ExperienceEntry(entry: &'static Experience<'static>) -> impl IntoView {
    view! {
        <article class="timeline-entry">
            <div class="timeline-entry__marker" aria-hidden="true"></div>
            <div class="timeline-entry__content">
                <p class="timeline-entry__period">{format_date_range(entry.dates)}</p>
                <h3><RichTextView text=&entry.role /></h3>
                <p class="timeline-entry__organisation">
                    <RichTextView text=&entry.organisation />
                    <span aria-hidden="true">" · "</span>
                    {format_location(&entry.location)}
                </p>
                {(!entry.highlights.is_empty()).then(|| view! {
                    <RichTextList items=&entry.highlights label="Responsibilities and achievements" />
                })}
            </div>
        </article>
    }
}

#[component]
fn EducationSection(entries: &'static [Education<'static>]) -> impl IntoView {
    view! {
        <section class="section section--surface cv-section">
            <Container>
                <div class="cv-layout">
                    <SectionHeading eyebrow="Academic background" title="Education" />
                    <div class="timeline">
                        {entries.iter().map(|entry| view! { <EducationEntry entry=entry /> }).collect_view()}
                    </div>
                </div>
            </Container>
        </section>
    }
}

#[component]
fn EducationEntry(entry: &'static Education<'static>) -> impl IntoView {
    view! {
        <article class="timeline-entry">
            <div class="timeline-entry__marker" aria-hidden="true"></div>
            <div class="timeline-entry__content">
                <p class="timeline-entry__period">{format_date_range(entry.dates)}</p>
                <h3><RichTextView text=&entry.qualification /></h3>
                <p class="timeline-entry__organisation">
                    <RichTextView text=&entry.institution />
                    <span aria-hidden="true">" · "</span>
                    {format_location(&entry.location)}
                </p>
            </div>
        </article>
    }
}

#[component]
fn SkillsSection(groups: &'static [SkillGroup<'static>]) -> impl IntoView {
    view! {
        <section class="section cv-section">
            <Container>
                <div class="cv-layout">
                    <SectionHeading eyebrow="Capabilities" title="Technical skills" />
                    <div class="skill-groups skill-groups--cv">
                        {groups.iter().map(|group| view! { <CvSkillGroup group=group /> }).collect_view()}
                    </div>
                </div>
            </Container>
        </section>
    }
}

#[component]
fn CvSkillGroup(group: &'static SkillGroup<'static>) -> impl IntoView {
    view! {
        <article class="skill-group">
            <h3>{group.category.as_ref()}</h3>
            {(!group.skills.is_empty()).then(|| view! {
                <ul class="skill-list" aria-label=format!("{} skills", group.category)>
                    {group.skills.iter().map(|skill| view! {
                        <li class="skill-badge">{skill.as_ref()}</li>
                    }).collect_view()}
                </ul>
            })}
        </article>
    }
}

#[component]
fn RichTextList(items: &'static [RichText<'static>], label: &'static str) -> impl IntoView {
    view! {
        <ul class="timeline-entry__highlights" aria-label=label>
            {items.iter().map(|item| view! {
                <li><RichTextView text=item /></li>
            }).collect_view()}
        </ul>
    }
}

/// Renders only the importer-supported inline nodes as typed Leptos views.
#[component]
pub fn RichTextView(text: &'static RichText<'static>) -> impl IntoView {
    text.nodes.iter().map(inline_view).collect_view()
}

fn inline_view(node: &'static Inline<'static>) -> AnyView {
    match node {
        Inline::Text(text) => text.as_ref().into_any(),
        Inline::Strong(content) => {
            view! { <strong><RichTextView text=content /></strong> }.into_any()
        }
        Inline::Emphasis(content) => view! { <em><RichTextView text=content /></em> }.into_any(),
        Inline::Underline(content) => view! { <u><RichTextView text=content /></u> }.into_any(),
        Inline::Link { target, label } => {
            let opens_new_tab = target.starts_with("https://");
            view! {
                <a
                    class="cv-inline-link"
                    href=target.as_ref()
                    target=opens_new_tab.then_some("_blank")
                    rel=opens_new_tab.then_some("noreferrer")
                >
                    <RichTextView text=label />
                    {opens_new_tab.then(|| view! {
                        <span class="sr-only">" (opens in a new tab)"</span>
                    })}
                </a>
            }
            .into_any()
        }
    }
}

#[component]
fn CvSourceVersion(tag: &'static str, commit_sha: &'static str) -> impl IntoView {
    view! {
        <section class="cv-source" aria-labelledby="cv-source-title">
            <Container>
                <div class="cv-source__content">
                    <div>
                        <p class="eyebrow">"Synchronized source"</p>
                        <h2 id="cv-source-title">"CV version " {tag}</h2>
                    </div>
                    <p>
                        "Generated from upstream commit "
                        <code title=commit_sha>{short_commit_sha(commit_sha)}</code>
                        "."
                    </p>
                </div>
            </Container>
        </section>
    }
}

#[component]
fn CvUnavailable(pdf_url: Option<&'static str>) -> impl IntoView {
    view! {
        <section class="page-hero cv-unavailable" aria-labelledby="cv-unavailable-title">
            <Container>
                <p class="eyebrow">"Curriculum vitae"</p>
                <h1 id="cv-unavailable-title">"CV temporarily unavailable"</h1>
                <p class="page-hero__lead" role="status">
                    "The generated CV data could not be displayed. Please try again after the next synchronized release."
                </p>
                {pdf_url.map(|url| view! {
                    <a class="button button--secondary" href=url download=CV_PDF_FILENAME>
                        "Download the PDF instead"
                    </a>
                })}
            </Container>
        </section>
    }
}

/// Formats a month-precision imported date range for visual presentation.
pub fn format_date_range(range: DateRange) -> String {
    let end = match range.end {
        DateRangeEnd::Date(date) => format_cv_date(date),
        DateRangeEnd::Present => "Present".to_owned(),
    };
    format!("{} – {end}", format_cv_date(range.start))
}

fn format_cv_date(date: CvDate) -> String {
    format!("{} {}", month_label(date.month), date.year)
}

fn month_label(month: Month) -> &'static str {
    match month {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    }
}

/// Formats an imported city and country without changing authored values.
pub fn format_location(location: &Location<'_>) -> String {
    format!("{}, {}", location.city, location.country)
}

fn platform_label(platform: SocialPlatform) -> &'static str {
    match platform {
        SocialPlatform::GitHub => "GitHub",
        SocialPlatform::LinkedIn => "LinkedIn",
    }
}

fn short_commit_sha(commit_sha: &str) -> &str {
    commit_sha.get(..7).unwrap_or(commit_sha)
}

/// Renders the browser's CV document to HTML for native integration tests.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_cv_document(
    cv: Option<&'static Cv<'static>>,
    source_tag: &'static str,
    source_commit_sha: &'static str,
    pdf_url: Option<&'static str>,
) -> String {
    CvDocument(CvDocumentProps {
        cv,
        source_tag,
        source_commit_sha,
        pdf_url,
    })
    .to_html()
}

/// Renders imported rich text through the production safe-inline component.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_rich_text(text: &'static RichText<'static>) -> String {
    RichTextView(RichTextViewProps { text }).to_html()
}
