//! Canonical public route metadata used by navigation and tests.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteInfo {
    pub path: &'static str,
    pub label: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

/// Stable site-wide metadata visible in the initial client-rendered document.
///
/// A public origin has not yet been selected, so this deliberately contains no
/// canonical URL or absolute sharing-asset URL. See `docs/web-quality.md` for
/// the deployment-time metadata contract.
pub const SITE_NAME: &str = "Hayden Farrell";
pub const SITE_DESCRIPTION: &str =
    "Hayden Farrell - software engineer, selected projects and professional CV.";
pub const HOME: RouteInfo = RouteInfo {
    path: "/",
    label: "Home",
    title: "Hayden Farrell | Software Engineer",
    description: "Hayden Farrell - software engineer and computer science student.",
};
pub const PROJECTS: RouteInfo = RouteInfo {
    path: "/projects",
    label: "Projects",
    title: "Projects | Hayden Farrell",
    description: "Selected software engineering projects and technical case studies.",
};
pub const CV: RouteInfo = RouteInfo {
    path: "/cv",
    label: "CV",
    title: "CV | Hayden Farrell",
    description: "Hayden Farrell's generated curriculum vitae: experience, education, projects and technical skills.",
};
pub const NAVIGATION_ROUTES: &[RouteInfo] = &[HOME, PROJECTS, CV];

pub const NOT_FOUND: RouteInfo = RouteInfo {
    path: "/not-found",
    label: "Not found",
    title: "Page not found | Hayden Farrell",
    description: "The requested page could not be found. Return to Hayden Farrell's software engineering portfolio.",
};

pub fn metadata_for_path(path: &str) -> RouteInfo {
    NAVIGATION_ROUTES
        .iter()
        .find(|route| route.path == path)
        .copied()
        .unwrap_or(NOT_FOUND)
}

pub fn title_for_path(path: &str) -> &'static str {
    metadata_for_path(path).title
}
