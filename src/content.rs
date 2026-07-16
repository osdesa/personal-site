//! Central, type-safe portfolio content.
//!
//! Replace the clearly marked sample values in [`portfolio`] with real personal
//! information. Presentation components deliberately do not own content.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: &'static str,
    pub initials: &'static str,
    pub role: &'static str,
    pub eyebrow: &'static str,
    pub headline: &'static str,
    pub home_intro: &'static str,
    pub summary: &'static str,
    pub location: &'static str,
    pub availability: &'static str,
    pub email: &'static str,
    pub cv_download_url: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SocialLink {
    pub label: &'static str,
    pub url: &'static str,
    pub description: &'static str,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillGroup {
    pub category: &'static str,
    pub summary: &'static str,
    pub skills: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimelineItem {
    pub title: &'static str,
    pub organisation: &'static str,
    pub period: &'static str,
    pub summary: &'static str,
    pub tags: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub struct Portfolio {
    pub profile: Profile,
    pub social_links: &'static [SocialLink],
    pub projects: &'static [Project],
    pub skills: &'static [SkillGroup],
    pub experience: &'static [TimelineItem],
    pub education: &'static [TimelineItem],
}

const SOCIAL_LINKS: &[SocialLink] = &[
    SocialLink {
        label: "GitHub",
        url: "https://github.com/osdesa",
        description: "View Hayden's code on GitHub",
    },
    SocialLink {
        label: "LinkedIn",
        url: "https://www.linkedin.com/in/haydenfarrell",
        description: "Connect with Hayden on LinkedIn",
    },
    SocialLink {
        label: "Email",
        url: "mailto:haydenfarrell@outlook.com",
        description: "Email Hayden Farrell",
    },
];

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

const SKILLS: &[SkillGroup] = &[
    SkillGroup {
        category: "Languages",
        summary: "Replace this sample group with languages you use confidently.",
        skills: &["Rust", "TypeScript", "Python", "SQL"],
    },
    SkillGroup {
        category: "Engineering",
        summary: "Example disciplines that can be adjusted to match your experience.",
        skills: &["System design", "Testing", "APIs", "Accessibility"],
    },
    SkillGroup {
        category: "Tools & platforms",
        summary: "Keep only tools that reflect your real working knowledge.",
        skills: &["Git", "Linux", "Docker", "GitHub Actions"],
    },
];

const EXPERIENCE: &[TimelineItem] = &[
    TimelineItem {
        title: "Example software engineering role",
        organisation: "Replace with organisation",
        period: "20XX — Present",
        summary: "Sample copy: describe the scope of the role, the systems you helped improve and the evidence that best demonstrates your contribution.",
        tags: &["Ownership", "Collaboration", "Delivery"],
    },
    TimelineItem {
        title: "Example earlier role or placement",
        organisation: "Replace with organisation",
        period: "20XX — 20XX",
        summary: "Sample copy: explain what you learned, how you worked with others and which technical responsibilities are most relevant now.",
        tags: &["Learning", "Quality", "Teamwork"],
    },
];

const EDUCATION: &[TimelineItem] = &[TimelineItem {
    title: "Example degree or qualification",
    organisation: "Replace with institution",
    period: "20XX — 20XX",
    summary: "Sample copy: add the real subject, classification where appropriate, and a concise note about relevant study or a major project.",
    tags: &["Computer science", "Software engineering"],
}];

/// Returns all editable site content from one obvious source.
pub const fn portfolio() -> Portfolio {
    Portfolio {
        profile: Profile {
            name: "Hayden Farrell",
            initials: "HF",
            role: "Software Engineer",
            eyebrow: "Portfolio",
            headline: "Software engineer building dependable, thoughtful digital products.",
            home_intro: "Software engineer and computer science student.",
            summary: "I care about clear systems, accessible interfaces and software that remains easy to change. This polished foundation is ready for your real story.",
            location: "Your location • Replace me",
            availability: "Availability status • Replace me",
            email: "haydenfarrell@outlook.com",
            cv_download_url: Some("/cv/Hayden-Farrell-CV.pdf"),
        },
        social_links: SOCIAL_LINKS,
        projects: PROJECTS,
        skills: SKILLS,
        experience: EXPERIENCE,
        education: EDUCATION,
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
