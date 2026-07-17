//! Central editorial content independent of generated CV and project data.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub role: &'static str,
    pub home_intro: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct Portfolio {
    pub profile: Profile,
}

/// Returns homepage-specific editorial content.
pub const fn portfolio() -> Portfolio {
    Portfolio {
        profile: Profile {
            role: "Software Engineer",
            home_intro: "Computer Science student and part-time software engineer working across safety-critical systems, C++, and GPU computing.",
        },
    }
}
