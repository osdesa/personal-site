use std::collections::HashSet;

use personal_site::content::{featured_projects, portfolio, project_by_id};

#[test]
fn project_ids_are_unique_and_url_safe() {
    let projects = portfolio().projects;
    let unique_ids: HashSet<_> = projects.iter().map(|project| project.id).collect();

    assert_eq!(unique_ids.len(), projects.len());
    assert!(projects.iter().all(|project| {
        !project.id.is_empty()
            && project.id.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
    }));
}

#[test]
fn projects_have_required_content_and_secure_links() {
    for project in portfolio().projects {
        assert!(!project.title.trim().is_empty());
        assert!(!project.description.trim().is_empty());
        assert!(!project.technologies.is_empty());
        assert!(project.repository_url.starts_with("https://"));
        assert!(
            project
                .demo_url
                .is_none_or(|url| url.starts_with("https://"))
        );
    }
}

#[test]
fn featured_projects_are_discoverable_by_id() {
    let featured: Vec<_> = featured_projects().collect();

    assert!(!featured.is_empty());
    assert!(featured.iter().all(|project| project.featured));
    assert!(
        featured
            .iter()
            .all(|project| project_by_id(project.id) == Some(*project))
    );
}

#[test]
fn contact_data_is_complete_and_centralised() {
    let content = portfolio();

    assert!(content.profile.email.contains('@'));
    assert!(!content.profile.role.is_empty());
    assert!(!content.profile.home_intro.is_empty());
    assert_eq!(content.social_links.len(), 3);
    assert!(content.social_links.iter().all(|link| !link.url.is_empty()));
}

#[test]
fn cv_content_covers_each_professional_area() {
    let content = portfolio();

    assert_eq!(content.experience.len(), 2);
    assert_eq!(content.education.len(), 2);
    assert_eq!(content.skills.len(), 5);

    for item in content.experience.iter().chain(content.education) {
        assert!(!item.title.trim().is_empty());
        assert!(!item.organisation.trim().is_empty());
        assert!(!item.location.trim().is_empty());
        assert!(!item.period.trim().is_empty());
        assert!(!item.summary.trim().is_empty());
        assert!(!item.tags.is_empty());
        assert!(
            item.highlights
                .iter()
                .all(|highlight| !highlight.trim().is_empty())
        );
    }
}

#[test]
fn published_cv_content_does_not_contain_placeholder_markers() {
    let content = portfolio();
    let timeline_text = content
        .experience
        .iter()
        .chain(content.education)
        .flat_map(|item| [item.title, item.organisation, item.summary]);

    for value in timeline_text {
        let value = value.to_ascii_lowercase();
        assert!(!value.contains("placeholder"));
        assert!(!value.contains("replace me"));
        assert!(!value.contains("example:"));
        assert!(!value.contains("20xx"));
    }
}
