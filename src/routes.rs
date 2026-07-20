//! Canonical public route metadata used by navigation and tests.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteInfo {
    pub path: &'static str,
    pub label: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

/// Typed, canonical HTTPS origin for every production absolute URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiteOrigin(&'static str);

impl SiteOrigin {
    /// Returns the canonical origin without a trailing slash.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// Creates an absolute URL from a root-relative public path.
    #[must_use]
    pub fn absolute_url(self, path: &str) -> String {
        debug_assert!(path.starts_with('/'));
        format!("{}{}", self.0, path)
    }
}

/// The single source of truth for the site's production origin.
pub const PRODUCTION_ORIGIN: SiteOrigin = SiteOrigin("https://haydenfarrell.dev");

/// Controlled local artwork used as the generic sharing image.
pub const SOCIAL_IMAGE_PATH: &str = "/images/project-default.svg";

/// Stable site-wide metadata visible in the initial client-rendered document.
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

/// Returns the canonical absolute URL for a known public route.
#[must_use]
pub fn canonical_url_for_path(path: &str) -> String {
    PRODUCTION_ORIGIN.absolute_url(metadata_for_path(path).path)
}

/// Returns the absolute URL of the generic sharing artwork.
#[must_use]
pub fn social_image_url() -> String {
    PRODUCTION_ORIGIN.absolute_url(SOCIAL_IMAGE_PATH)
}
