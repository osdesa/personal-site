//! Central, type-safe portfolio content.
//!
//! Presentation components deliberately do not own content. Update this module
//! when professional details, projects, or the downloadable CV change.

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
    pub location: &'static str,
    pub period: &'static str,
    pub summary: &'static str,
    pub highlights: &'static [&'static str],
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
        summary: "Programming languages and build languages used across systems, embedded, academic, and tooling work.",
        skills: &[
            "C++",
            "C",
            "Python",
            "Java",
            "MATLAB",
            "VHDL",
            "ARM",
            "Rust",
            "Shell scripting",
            "CMake",
        ],
    },
    SkillGroup {
        category: "Libraries",
        summary: "Graphics, GPU compute, and web libraries used to build technical applications.",
        skills: &["OpenGL", "Vulkan", "Flask"],
    },
    SkillGroup {
        category: "Tools & practices",
        summary: "Development, continuous integration, and safety-critical engineering tools and practices.",
        skills: &[
            "Git",
            "Docker",
            "VS Code",
            "Visual Studio",
            "IntelliJ",
            "Jenkins",
            "MISRA",
            "Polyspace",
        ],
    },
    SkillGroup {
        category: "Training & certificates",
        summary: "Formal training supporting safety-critical and modern C++ development.",
        skills: &["DO-178C certified", "C++ training"],
    },
    SkillGroup {
        category: "Interests",
        summary: "Interests outside day-to-day software engineering.",
        skills: &["Climbing & bouldering", "Leadership", "Drones"],
    },
];

const EXPERIENCE: &[TimelineItem] = &[
    TimelineItem {
        title: "Part-time Software Engineer",
        organisation: "Leonardo UK",
        location: "Newcastle upon Tyne, UK · Remote",
        period: "July 2026 — Present",
        summary: "One of three undergraduate software engineers offered a part-time role while returning to university.",
        highlights: &[
            "Develop safety-critical real-time software as part of an Agile engineering team while completing a Computer Science degree.",
            "Balance university and work commitments to meet deadlines across both roles.",
            "Maintain and monitor Jenkins CI pipelines to support reliable build and test execution alongside ongoing development.",
        ],
        tags: &["Real-time software", "Jenkins", "Agile", "CI"],
    },
    TimelineItem {
        title: "Industrial Placement Software Engineer",
        organisation: "Leonardo UK",
        location: "Newcastle upon Tyne, UK",
        period: "June 2025 — July 2026",
        summary: "Developed and maintained safety-critical real-time software within an Agile engineering team following DO-178C development practices.",
        highlights: &[
            "Led the design and implementation of a C++ synthetic testing framework for complex embedded software, reducing dependency on hardware integration during development.",
            "Led design workshops with more than 10 engineers, including principal and lead engineers, to gather requirements and validate the software design.",
            "Developed a custom test application used by more than 10 engineers across teams for rapid developer testing of a complex engineering system.",
            "Created a tool for visualising large volumes of real-time mathematical data, enabling faster analysis of complex engineering systems.",
        ],
        tags: &["C++", "DO-178C", "Embedded systems", "Developer tooling"],
    },
];

const EDUCATION: &[TimelineItem] = &[
    TimelineItem {
        title: "BSc Computer Science with a Year in Industry",
        organisation: "University of Nottingham",
        location: "Nottingham, UK",
        period: "August 2023 — May 2027",
        summary: "Predicted First-Class Honours, including an industrial placement year in software engineering.",
        highlights: &[],
        tags: &["Computer Science", "Predicted First-Class Honours"],
    },
    TimelineItem {
        title: "A levels",
        organisation: "Aquinas College",
        location: "Manchester, UK",
        period: "August 2021 — May 2023",
        summary: "Mathematics (A), Computer Science (A), and Further Mathematics (A).",
        highlights: &[],
        tags: &["Mathematics", "Computer Science", "Further Mathematics"],
    },
];

/// Returns all editable site content from one obvious source.
pub const fn portfolio() -> Portfolio {
    Portfolio {
        profile: Profile {
            name: "Hayden Farrell",
            initials: "HF",
            role: "Software Engineer",
            eyebrow: "Portfolio",
            headline: "Software engineer building dependable real-time systems and developer tooling.",
            home_intro: "Computer Science student and part-time software engineer working across safety-critical systems, C++, and GPU computing.",
            summary: "I build safety-critical real-time software at Leonardo UK while studying Computer Science at the University of Nottingham. My experience spans embedded C++, developer tooling, continuous integration, GPU compute, and testing infrastructure, with an emphasis on dependable systems and practical engineering outcomes.",
            location: "United Kingdom",
            availability: "Graduating May 2027",
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
