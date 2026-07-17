//! Central, type-safe content that is independent of the generated CV.
//!
//! Identity, contact details and CV sections come from `generated_cv`. This
//! module owns only page-specific editorial copy and the separate portfolio
//! project catalogue.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub role: &'static str,
    pub home_intro: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Project {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub technologies: &'static [&'static str],
    pub repository_url: &'static str,
    pub demo_url: Option<&'static str>,
    pub image_url: Option<&'static str>,
    pub featured: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Portfolio {
    pub profile: Profile,
    pub projects: &'static [Project],
}

const PROJECTS: &[Project] = &[
    Project {
        id: "example-systems-dashboard",
        title: "Example: Systems dashboard",
        description: "A sample portfolio entry showing how a production service, its technical decisions and outcomes can be presented without crowding the page.",
        technologies: &["Rust", "Leptos", "Observability"],
        repository_url: "https://github.com/your-username/example-systems-dashboard",
        demo_url: Some("https://example.com"),
        image_url: None,
        featured: true,
    },
    Project {
        id: "example-developer-tool",
        title: "Example: Developer tool",
        description: "A clearly labelled sample for a focused command-line or productivity tool, ready to be replaced with a real project and measurable context.",
        technologies: &["Rust", "CLI", "Testing"],
        repository_url: "https://github.com/your-username/example-developer-tool",
        demo_url: None,
        image_url: None,
        featured: true,
    },
    Project {
        id: "example-web-platform",
        title: "Example: Web platform",
        description: "A placeholder case study for an accessible, responsive web product with a maintainable component architecture and a considered user experience.",
        technologies: &["Leptos", "Tailwind CSS", "Accessibility"],
        repository_url: "https://github.com/your-username/example-web-platform",
        demo_url: Some("https://example.com"),
        image_url: None,
        featured: true,
    },
    Project {
        id: "example-university-project",
        title: "Example: University project",
        description: "A sample location for a substantial academic project. Replace this with the problem, approach, individual contribution and evidence from the real work.",
        technologies: &["Algorithms", "Research", "Documentation"],
        repository_url: "https://github.com/your-username/example-university-project",
        demo_url: None,
        image_url: None,
        featured: false,
    },
];

/// Returns the hand-authored content that is not imported from the CV.
pub const fn portfolio() -> Portfolio {
    Portfolio {
        profile: Profile {
            role: "Software Engineer",
            home_intro: "Computer Science student and part-time software engineer working across safety-critical systems, C++, and GPU computing.",
        },
        projects: PROJECTS,
    }
}

/// Finds a project by its stable identifier.
pub fn project_by_id(id: &str) -> Option<Project> {
    portfolio()
        .projects
        .iter()
        .copied()
        .find(|project| project.id == id)
}

/// Returns projects marked for featured presentation surfaces.
pub fn featured_projects() -> impl Iterator<Item = Project> {
    portfolio()
        .projects
        .iter()
        .copied()
        .filter(|project| project.featured)
}
