//! Canonical public route metadata used by navigation and tests.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteInfo {
    pub path: &'static str,
    pub label: &'static str,
}

pub const HOME: RouteInfo = RouteInfo {
    path: "/",
    label: "Home",
};
pub const PROJECTS: RouteInfo = RouteInfo {
    path: "/projects",
    label: "Projects",
};
pub const CV: RouteInfo = RouteInfo {
    path: "/cv",
    label: "CV",
};
pub const NAVIGATION_ROUTES: &[RouteInfo] = &[HOME, PROJECTS, CV];

pub fn title_for_path(path: &str) -> &'static str {
    NAVIGATION_ROUTES
        .iter()
        .find(|route| route.path == path)
        .map_or("Page not found | Hayden Farrell", |route| {
            match route.path {
                "/" => "Hayden Farrell | Software Engineer",
                "/projects" => "Projects | Hayden Farrell",
                "/cv" => "CV | Hayden Farrell",
                _ => "Hayden Farrell | Software Engineer",
            }
        })
}
