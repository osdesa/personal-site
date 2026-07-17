use std::collections::HashSet;

use personal_site::content::portfolio;
use personal_site::generated_cv::CV;
use personal_site::generated_projects::PROJECTS;
use personal_site::projects::project_by_id;

#[test]
fn generated_project_ids_are_unique_and_url_safe() {
    let unique_ids: HashSet<_> = PROJECTS.iter().map(|project| project.id).collect();

    assert!(!PROJECTS.is_empty());
    assert!(PROJECTS.len() <= 4);
    assert_eq!(unique_ids.len(), PROJECTS.len());
    assert!(PROJECTS.iter().all(|project| {
        !project.id.is_empty()
            && project.id.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
    }));
}

#[test]
fn generated_projects_have_complete_safe_presentation_data() {
    for project in PROJECTS {
        assert!(!project.title.trim().is_empty());
        assert!(!project.summary.trim().is_empty());
        assert!(!project.technologies.is_empty());
        assert!(project.image_url.starts_with('/') || project.image_url.starts_with("https://"));
        assert!(
            project
                .repository_url
                .is_none_or(|url| url.starts_with("https://"))
        );
        assert!(
            project
                .demo_url
                .is_none_or(|url| url.starts_with("https://"))
        );
        assert_eq!(project_by_id(project.id), Some(*project));
    }
}

#[test]
fn contact_data_is_complete_and_centralised() {
    let content = portfolio();

    assert!(!content.profile.role.is_empty());
    assert!(!content.profile.home_intro.is_empty());
    assert!(CV.profile.contact.email.contains('@'));
    assert!(!CV.profile.full_name.is_empty());
    assert!(!CV.profile.social_links.is_empty());
}
