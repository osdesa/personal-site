use personal_site::{
    cv::{DateRangeEnd, Inline, Month, SocialPlatform},
    cv_sync::{RemoteTag, generate_cv_module, parse_cv},
    generated_cv,
};

const SOURCE: &str = include_str!("../public/cv/Hayden-Farrell-CV.tex");
const GENERATED: &[u8] = include_bytes!("../src/generated_cv.rs");
const SHA: &str = "5c689c5fc89c9121a00ff2260dd48d2feef6c0ac";

fn source_identity() -> RemoteTag {
    RemoteTag {
        name: "v1.0.0".to_owned(),
        commit_sha: SHA.to_owned(),
    }
}

#[test]
fn checked_in_cv_parses_every_supported_domain_section() {
    let cv = parse_cv(SOURCE).unwrap();

    assert_eq!(cv.profile.full_name, "Hayden Farrell");
    assert_eq!(cv.profile.contact.email, "haydenfarrell@outlook.com");
    assert_eq!(cv.profile.social_links.len(), 2);
    assert_eq!(
        cv.profile.social_links[0].platform,
        SocialPlatform::LinkedIn
    );
    assert_eq!(cv.profile.social_links[1].platform, SocialPlatform::GitHub);

    assert_eq!(cv.education.len(), 2);
    assert_eq!(cv.education[0].location.city, "Nottingham");
    assert_eq!(cv.education[0].location.country, "UK");
    assert_eq!(cv.education[0].dates.start.month, Month::August);
    assert_eq!(cv.education[0].dates.start.year, 2023);

    assert_eq!(cv.experience.len(), 2);
    assert_eq!(cv.experience[0].highlights.len(), 4);
    assert_eq!(cv.experience[1].highlights.len(), 5);
    assert_eq!(cv.experience[0].dates.end, DateRangeEnd::Present);
    assert_eq!(cv.experience[1].dates.start.month, Month::June);

    assert_eq!(cv.projects.len(), 2);
    assert_eq!(cv.projects[0].technologies.len(), 4);
    assert_eq!(cv.projects[0].period.as_ref().unwrap().nodes.len(), 1);
    assert_eq!(cv.projects[1].highlights.len(), 3);

    assert_eq!(cv.skills.len(), 5);
    assert_eq!(cv.skills[0].category, "Languages");
    assert_eq!(cv.skills[0].skills[0], "C++");
    assert_eq!(cv.skills[4].category, "Hobbies");
}

#[test]
fn nested_project_link_formatting_is_structured_not_html() {
    let cv = parse_cv(SOURCE).unwrap();
    let Inline::Strong(strong) = &cv.projects[1].title.nodes[0] else {
        panic!("project title should preserve textbf as Strong");
    };
    let Inline::Link { target, label } = &strong.nodes[0] else {
        panic!("Blocky title should preserve href as Link");
    };
    assert_eq!(target, "https://github.com/osdesa/Blocky");
    let Inline::Underline(underlined) = &label.nodes[0] else {
        panic!("Blocky link label should preserve underline formatting");
    };
    assert_eq!(underlined.nodes.as_ref(), &[Inline::Text("Blocky".into())]);
}

#[test]
fn unknown_inline_commands_fail_with_clear_source_diagnostics() {
    let invalid = SOURCE.replacen(
        "Developing a user-space",
        "\\unknown{value} Developing a user-space",
        1,
    );

    let error = parse_cv(&invalid).unwrap_err();

    assert!(error.line() > 100);
    assert!(error.column() > 0);
    assert!(
        error
            .message()
            .contains("unsupported inline command \\unknown")
    );
    assert!(error.to_string().starts_with("line "));
}

#[test]
fn malformed_groups_and_missing_required_sections_are_rejected() {
    let malformed = SOURCE.replacen(
        "\\resumeItem{Developing a user-space",
        "\\resumeItem Developing a user-space",
        1,
    );
    let malformed_error = parse_cv(&malformed).unwrap_err();
    assert!(malformed_error.message().contains("expected '{'"));

    let missing = SOURCE.replacen("\\section{Education}", "\\section{Other}", 1);
    let missing_error = parse_cv(&missing).unwrap_err();
    assert!(
        missing_error
            .message()
            .contains("expected section \"Education\"")
    );
}

#[test]
fn missing_custom_command_declarations_are_rejected_before_body_parsing() {
    let invalid = SOURCE.replacen("\\newcommand{\\resumeItem}[1]", "", 1);

    let error = parse_cv(&invalid).unwrap_err();

    assert!(error.message().contains("resumeItem"));
    assert!(error.message().contains("preamble"));
}

#[test]
fn malformed_required_semantic_values_are_rejected() {
    let bad_location = SOURCE.replacen("{Nottingham, UK}", "{Nottingham}", 1);
    assert!(
        parse_cv(&bad_location)
            .unwrap_err()
            .message()
            .contains("City, Country")
    );

    let bad_date = SOURCE.replacen("{Aug. 2023 -- May 2027}", "{Someday -- May 2027}", 1);
    assert!(
        parse_cv(&bad_date)
            .unwrap_err()
            .message()
            .contains("malformed month/year")
    );

    let missing_email = SOURCE.replacen("mailto:haydenfarrell@outlook.com", "mailto:", 1);
    assert!(
        parse_cv(&missing_email)
            .unwrap_err()
            .message()
            .contains("email address is malformed")
    );
}

#[test]
fn parsing_and_generation_are_deterministic_regression_boundaries() {
    let first = parse_cv(SOURCE).unwrap();
    let second = parse_cv(SOURCE).unwrap();
    assert_eq!(first, second);

    let first_generated = generate_cv_module(&first, &source_identity());
    let second_generated = generate_cv_module(&second, &source_identity());
    assert_eq!(first_generated, second_generated);
    if first_generated != GENERATED {
        let mismatch = first_generated
            .iter()
            .zip(GENERATED)
            .position(|(left, right)| left != right)
            .unwrap_or(first_generated.len().min(GENERATED.len()));
        panic!(
            "generated regression differs at byte {mismatch}: expected {:?}, actual {:?}",
            String::from_utf8_lossy(
                &first_generated
                    [mismatch.saturating_sub(40)..(mismatch + 80).min(first_generated.len())]
            ),
            String::from_utf8_lossy(
                &GENERATED[mismatch.saturating_sub(40)..(mismatch + 80).min(GENERATED.len())]
            )
        );
    }
    assert_eq!(generated_cv::CV, first);
    assert_eq!(generated_cv::SOURCE_TAG, "v1.0.0");
    assert_eq!(generated_cv::SOURCE_COMMIT_SHA, SHA);
}
