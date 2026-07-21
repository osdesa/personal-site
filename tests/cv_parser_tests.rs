use personal_site::{
    cv::{Inline, SocialPlatform},
    cv_sync::{RemoteTag, generate_cv_module, parse_cv},
    generated_cv,
};

const SOURCE: &str = include_str!("../public/cv/Hayden-Farrell-CV.tex");
const GENERATED: &[u8] = include_bytes!("../src/generated_cv.rs");
const COMMIT_SHA_LENGTH: usize = 40;

fn source_identity() -> RemoteTag {
    RemoteTag {
        name: generated_cv::SOURCE_TAG.to_owned(),
        commit_sha: generated_cv::SOURCE_COMMIT_SHA.to_owned(),
    }
}

fn replace_first_document_content(source: &str, pattern: &str, replacement: &str) -> String {
    let document_start = source
        .find("\\begin{document}")
        .expect("the supported CV grammar requires a document body");
    let content_start = document_start + "\\begin{document}".len();
    let match_start = content_start
        + source[content_start..]
            .find(pattern)
            .expect("the supported CV grammar requires the requested content");

    format!(
        "{}{}{}",
        &source[..match_start],
        replacement,
        &source[match_start + pattern.len()..]
    )
}

fn replace_first_education_argument(
    source: &str,
    argument_index: usize,
    replacement: &str,
) -> String {
    let education_start = source
        .find("\\section{Education}")
        .expect("the supported CV grammar requires an Education section");
    let entry_start = education_start
        + source[education_start..]
            .find("\\resumeSubheading")
            .expect("the Education section requires an entry");
    let mut cursor = entry_start + "\\resumeSubheading".len();

    for index in 0..=argument_index {
        cursor += source[cursor..]
            .find('{')
            .expect("education entry arguments must be braced");
        let argument_start = cursor;
        let mut depth = 0;

        for (offset, character) in source[cursor..].char_indices() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let argument_end = cursor + offset + 1;
                        if index == argument_index {
                            return format!(
                                "{}{}{}",
                                &source[..argument_start],
                                replacement,
                                &source[argument_end..]
                            );
                        }
                        cursor = argument_end;
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    unreachable!("the requested education argument must exist")
}

fn replace_mailto_target(source: &str, replacement: &str) -> String {
    let target_start = source
        .find("mailto:")
        .expect("the supported CV grammar requires an email link");
    let target_end = target_start
        + source[target_start..]
            .find('}')
            .expect("the email link target must be braced");

    format!(
        "{}{}{}",
        &source[..target_start],
        replacement,
        &source[target_end..]
    )
}

#[test]
fn checked_in_cv_parses_every_supported_domain_section() {
    let cv = parse_cv(SOURCE).unwrap();

    assert!(!cv.profile.full_name.trim().is_empty());
    assert!(cv.profile.contact.email.contains('@'));
    assert_eq!(cv.profile.social_links.len(), 2);
    assert!(
        cv.profile
            .social_links
            .iter()
            .any(|link| link.platform == SocialPlatform::LinkedIn)
    );
    assert!(
        cv.profile
            .social_links
            .iter()
            .any(|link| link.platform == SocialPlatform::GitHub)
    );

    assert!(!cv.education.is_empty());
    assert!(cv.education.iter().all(|entry| {
        !entry.institution.nodes.is_empty()
            && !entry.qualification.nodes.is_empty()
            && !entry.location.city.trim().is_empty()
            && !entry.location.country.trim().is_empty()
    }));

    assert!(!cv.experience.is_empty());
    assert!(cv.experience.iter().all(|entry| {
        !entry.role.nodes.is_empty()
            && !entry.organisation.nodes.is_empty()
            && !entry.location.city.trim().is_empty()
            && !entry.location.country.trim().is_empty()
            && !entry.highlights.is_empty()
    }));

    assert!(!cv.projects.is_empty());
    assert!(cv.projects.iter().all(|entry| {
        !entry.title.nodes.is_empty()
            && !entry.technologies.is_empty()
            && !entry.highlights.is_empty()
    }));

    assert!(!cv.skills.is_empty());
    assert!(
        cv.skills
            .iter()
            .all(|group| !group.category.trim().is_empty() && !group.skills.is_empty())
    );
}

#[test]
fn nested_inline_formatting_is_structured_not_html() {
    let source = replace_first_document_content(
        SOURCE,
        "\\resumeItem{",
        "\\resumeItem{\\textbf{\\href{https://example.invalid}{\\underline{label}}}",
    );
    let cv = parse_cv(&source).unwrap();
    let Inline::Strong(strong) = &cv.experience[0].highlights[0].nodes[0] else {
        panic!("nested formatting should preserve textbf as Strong");
    };
    let Inline::Link { target, label } = &strong.nodes[0] else {
        panic!("nested formatting should preserve href as Link");
    };
    assert_eq!(target, "https://example.invalid");
    let Inline::Underline(underlined) = &label.nodes[0] else {
        panic!("nested formatting should preserve underline formatting");
    };
    assert_eq!(underlined.nodes.as_ref(), &[Inline::Text("label".into())]);
}

#[test]
fn unknown_inline_commands_fail_with_clear_source_diagnostics() {
    let invalid =
        replace_first_document_content(SOURCE, "\\resumeItem{", "\\resumeItem{\\unknown{value}");

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
    let malformed = replace_first_document_content(SOURCE, "\\resumeItem{", "\\resumeItem ");
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
    let bad_location = replace_first_education_argument(SOURCE, 1, "{Location}");
    assert!(
        parse_cv(&bad_location)
            .unwrap_err()
            .message()
            .contains("City, Country")
    );

    let bad_date = replace_first_education_argument(SOURCE, 3, "{Someday -- Present}");
    assert!(
        parse_cv(&bad_date)
            .unwrap_err()
            .message()
            .contains("malformed month/year")
    );

    let missing_email = replace_mailto_target(SOURCE, "mailto:");
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
    assert!(!generated_cv::SOURCE_TAG.is_empty());
    assert_eq!(generated_cv::SOURCE_COMMIT_SHA.len(), COMMIT_SHA_LENGTH);
    assert!(
        generated_cv::SOURCE_COMMIT_SHA
            .chars()
            .all(|character| { character.is_ascii_hexdigit() && !character.is_ascii_uppercase() })
    );
}
