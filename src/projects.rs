//! Presentation-independent portfolio project data.

/// Visibility reported by GitHub when the project data was synchronized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectVisibility {
    Public,
    Private,
}

/// A normalized project ready for static presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Project {
    pub id: &'static str,
    pub repository: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub visibility: ProjectVisibility,
    pub project_date: &'static str,
    pub status: Option<&'static str>,
    pub technologies: &'static [&'static str],
    pub highlights: &'static [&'static str],
    pub image_url: &'static str,
    pub repository_url: Option<&'static str>,
    pub demo_url: Option<&'static str>,
}

/// Returns a project by its stable, repository-derived identifier.
#[must_use]
pub fn project_by_id(id: &str) -> Option<Project> {
    crate::generated_projects::PROJECTS
        .iter()
        .copied()
        .find(|project| project.id == id)
}
