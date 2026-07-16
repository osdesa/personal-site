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
