use std::borrow::Cow;

use personal_site::cv::{
    ContactDetails, Cv, CvDate, DateRange, DateRangeEnd, Inline, Location, Month, Profile, RichText,
};
use personal_site::cv_presentation::{
    CV_PDF_FILENAME, CV_PDF_URL, format_date_range, format_location, render_cv_document,
    render_rich_text,
};
use personal_site::generated_cv::{CV, SOURCE_COMMIT_SHA, SOURCE_TAG};

static EMPTY_CV: Cv<'static> = Cv {
    profile: Profile {
        full_name: Cow::Borrowed("Test Person"),
        contact: ContactDetails {
            email: Cow::Borrowed("test@example.com"),
        },
        social_links: Cow::Borrowed(&[]),
    },
    education: Cow::Borrowed(&[]),
    experience: Cow::Borrowed(&[]),
    projects: Cow::Borrowed(&[]),
    skills: Cow::Borrowed(&[]),
};

static SAFE_INLINE_TEXT: RichText<'static> = RichText {
    nodes: Cow::Borrowed(&[
        Inline::Text(Cow::Borrowed("<script>alert('x')</script> ")),
        Inline::Strong(RichText {
            nodes: Cow::Borrowed(&[Inline::Text(Cow::Borrowed("strong"))]),
        }),
        Inline::Text(Cow::Borrowed(" ")),
        Inline::Emphasis(RichText {
            nodes: Cow::Borrowed(&[Inline::Text(Cow::Borrowed("emphasis"))]),
        }),
        Inline::Text(Cow::Borrowed(" ")),
        Inline::Underline(RichText {
            nodes: Cow::Borrowed(&[Inline::Text(Cow::Borrowed("underline"))]),
        }),
        Inline::Text(Cow::Borrowed(" ")),
        Inline::Link {
            target: Cow::Borrowed("mailto:test@example.com"),
            label: RichText {
                nodes: Cow::Borrowed(&[Inline::Text(Cow::Borrowed("email"))]),
            },
        },
        Inline::Text(Cow::Borrowed(" ")),
        Inline::Link {
            target: Cow::Borrowed("https://example.com/profile"),
            label: RichText {
                nodes: Cow::Borrowed(&[Inline::Text(Cow::Borrowed("profile"))]),
            },
        },
    ]),
};

fn render_generated_cv(pdf_url: Option<&'static str>) -> String {
    render_cv_document(Some(&CV), SOURCE_TAG, SOURCE_COMMIT_SHA, pdf_url)
}

#[test]
fn generated_stage_two_data_renders_every_major_cv_section() {
    let html = render_generated_cv(Some(CV_PDF_URL));

    for expected in [
        "Hayden Farrell",
        "haydenfarrell@outlook.com",
        "Professional experience",
        "Software Engineer part time",
        "Education",
        "University of Nottingham",
        "Technical skills",
        "Languages",
        "CV version",
        "v1.0.0",
        "5c689c5",
    ] {
        assert!(
            html.contains(expected),
            "missing rendered content: {expected}"
        );
    }

    assert!(html.contains("<address class=\"cv-contact-row\">"));
    assert!(html.contains("<article class=\"timeline-entry\">"));
}

#[test]
fn optional_sections_and_profile_links_are_omitted_when_empty() {
    let html = render_cv_document(Some(&EMPTY_CV), "v-test", "abcdef0", Some(CV_PDF_URL));

    assert!(html.contains("Test Person"));
    assert!(!html.contains("<h2>Professional experience</h2>"));
    assert!(!html.contains("<h2>Education</h2>"));
    assert!(!html.contains("<h2>Projects</h2>"));
    assert!(!html.contains("<h2>Technical skills</h2>"));
    assert!(!html.contains("Professional profiles"));
}

#[test]
fn generated_projects_are_deliberately_not_rendered_on_the_cv_page() {
    let html = render_generated_cv(Some(CV_PDF_URL));

    assert!(!CV.projects.is_empty());
    assert!(!html.contains("<h2>Projects</h2>"));
    assert!(!html.contains("Atlas"));
    assert!(!html.contains("Blocky"));
}

#[test]
fn generated_links_have_accessible_and_safe_behaviour() {
    let html = render_generated_cv(Some(CV_PDF_URL));

    assert!(html.contains("href=\"mailto:haydenfarrell@outlook.com\""));
    assert!(html.contains("aria-label=\"Email haydenfarrell@outlook.com\""));
    assert!(html.contains("target=\"_blank\""));
    assert!(html.contains("rel=\"noreferrer\""));
    assert!(html.contains("opens in a new tab"));
}

#[test]
fn pdf_link_downloads_the_synchronised_artifact_and_has_a_fallback() {
    let available = render_generated_cv(Some(CV_PDF_URL));
    assert!(available.contains(&format!("href=\"{CV_PDF_URL}\"")));
    assert!(available.contains(&format!("download=\"{CV_PDF_FILENAME}\"")));
    assert!(available.contains("aria-label=\"Download CV as a PDF\""));

    let unavailable = render_generated_cv(None);
    assert!(unavailable.contains("PDF unavailable"));
    assert!(unavailable.contains("role=\"status\""));
    assert!(unavailable.contains("aria-disabled=\"true\""));
    assert!(!unavailable.contains(&format!("href=\"{CV_PDF_URL}\"")));
}

#[test]
fn missing_generated_data_shows_a_useful_fallback() {
    let html = render_cv_document(None, "v-test", "abcdef0", Some(CV_PDF_URL));

    assert!(html.contains("CV temporarily unavailable"));
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("Download the PDF instead"));
    assert!(html.contains(&format!("href=\"{CV_PDF_URL}\"")));
    assert!(!html.contains("<h2>Professional experience</h2>"));
}

#[test]
fn supported_inline_formatting_is_structured_and_escaped() {
    let html = render_rich_text(&SAFE_INLINE_TEXT);

    assert!(html.contains("&lt;script&gt;alert('x')&lt;/script&gt;"));
    assert!(!html.contains("<script>"));
    assert!(html.contains("<strong>strong"));
    assert!(html.contains("<em>emphasis"));
    assert!(html.contains("<u>underline"));
    assert!(html.contains("href=\"mailto:test@example.com\""));
    assert!(html.contains("href=\"https://example.com/profile\""));
    assert!(html.contains("target=\"_blank\""));
}

#[test]
fn imported_dates_and_locations_have_consistent_presentation() {
    let dates = DateRange {
        start: CvDate {
            year: 2025,
            month: Month::June,
        },
        end: DateRangeEnd::Present,
    };
    let location = Location {
        city: Cow::Borrowed("Nottingham"),
        country: Cow::Borrowed("UK"),
    };

    assert_eq!(format_date_range(dates), "Jun 2025 – Present");
    assert_eq!(format_location(&location), "Nottingham, UK");
}

#[test]
fn cv_layout_has_mobile_and_tablet_adaptations_and_visible_focus() {
    let css = include_str!("../styles/input.css");

    assert!(css.contains("@media (max-width: 52rem)"));
    assert!(css.contains(".cv-layout"));
    assert!(css.contains(".timeline-entry"));
    assert!(css.contains("@media (max-width: 40rem)"));
    assert!(css.contains("grid-template-columns: 1fr"));
    assert!(css.contains(":focus-visible"));
    assert!(css.contains("outline: 3px solid var(--focus)"));
}
