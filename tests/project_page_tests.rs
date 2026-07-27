use leptos::prelude::*;
use personal_site::components::ProjectCard;
use personal_site::projects::{Project, ProjectVisibility};

#[test]
fn projects_render_through_the_shared_card_component() {
    let projects = [
        Project {
            id: "active-project",
            repository: "example-owner/active-project",
            title: "Active example",
            summary: "An active project supplied by the test.",
            visibility: ProjectVisibility::Public,
            project_date: "2030-01-01",
            status: Some("Active"),
            technologies: &["Example language"],
            highlights: &["An example highlight."],
            image_url: "/images/project-default.svg",
            repository_url: Some("https://example.test/active-project"),
            demo_url: None,
        },
        Project {
            id: "completed-project",
            repository: "example-owner/completed-project",
            title: "Completed example",
            summary: "A completed project supplied by the test.",
            visibility: ProjectVisibility::Public,
            project_date: "2029-01-01",
            status: Some("Completed"),
            technologies: &["Example tool"],
            highlights: &[],
            image_url: "/images/project-default.svg",
            repository_url: None,
            demo_url: Some("https://example.test/completed-project"),
        },
    ];
    let html = view! {
        <div>
            {projects.iter().map(|project| view! {
                <ProjectCard project=*project />
            }).collect_view()}
        </div>
    }
    .to_html();
    for project in projects {
        assert!(html.contains(project.title));
        assert!(html.contains(project.summary));
    }
    assert!(html.contains("project-card"));
    assert!(!html.contains("project-card__number"));
    assert!(html.contains("project-card__status--active"));
    assert!(html.contains("project-card__status--completed"));
}

#[test]
fn hidden_private_repository_has_indicator_and_no_broken_link() {
    let project = Project {
        id: "private",
        repository: "example-owner/private",
        title: "Private Project",
        summary: "Public portfolio description.",
        visibility: ProjectVisibility::Private,
        project_date: "2026-01-01",
        status: Some("Active"),
        technologies: &["Rust"],
        highlights: &["Private source remains private"],
        image_url: "/images/project-default.svg",
        repository_url: None,
        demo_url: None,
    };
    let html = view! { <ProjectCard project /> }.to_html();
    assert!(html.contains("Private repository"));
    assert!(!html.contains("href=\"\""));
    assert!(!html.contains(">Repository<"));
    assert!(html.contains("Private source remains private"));
}

#[test]
fn project_with_all_links_suppressed_does_not_render_an_empty_link_group() {
    let project = Project {
        id: "unlinked",
        repository: "example-owner/unlinked",
        title: "Unlinked Project",
        summary: "A project intentionally presented without external links.",
        visibility: ProjectVisibility::Public,
        project_date: "2026-01-01",
        status: None,
        technologies: &["Rust"],
        highlights: &[],
        image_url: "/images/project-default.svg",
        repository_url: None,
        demo_url: None,
    };
    let html = view! { <ProjectCard project /> }.to_html();
    assert!(!html.contains("project-card__links"));
}

#[test]
fn only_projects_page_consumes_the_generated_catalogue() {
    let home_source = include_str!("../src/pages/home.rs");
    let projects_source = include_str!("../src/pages/projects.rs");
    assert!(!home_source.contains("generated_projects::PROJECTS"));
    assert!(projects_source.contains("generated_projects::PROJECTS"));
    assert!(!projects_source.contains("src/content.rs"));
}

#[test]
fn project_and_mobile_navigation_ordinals_are_not_rendered() {
    let card_source = include_str!("../src/components/project_card.rs");
    let navigation_source = include_str!("../src/components/site_shell.rs");

    assert!(!card_source.contains("project-card__number"));
    assert!(!navigation_source.contains("format!(\"0{}\""));
}

#[test]
fn project_cards_keep_metadata_at_the_top_links_at_the_bottom_and_media_responsive() {
    let css = include_str!("../styles/input.css");

    assert!(css.contains("grid-template-rows: auto minmax(0, 1fr)"));
    assert!(css.contains(".project-card__links"));
    assert!(css.contains("margin-top: auto"));
    assert!(css.contains(".project-card h2"));
    assert!(css.contains("aspect-ratio: 608 / 272"));
    assert!(css.contains("min-height: 0"));
}

#[test]
fn projects_page_heading_uses_the_reduced_responsive_scale() {
    let css = include_str!("../styles/input.css");

    assert!(css.contains("font-size: clamp(3rem, 8vw, 6.5rem)"));
    assert!(!css.contains("font-size: clamp(3.5rem, 10vw, 8rem)"));
}
