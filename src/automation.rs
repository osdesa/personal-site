//! Shared, testable safety policy for generated-content pull requests.

/// The two automation sources allowed to request a merge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationKind {
    Cv,
    Projects,
}

impl AutomationKind {
    /// Parses a stable workflow argument.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "cv" => Ok(Self::Cv),
            "projects" => Ok(Self::Projects),
            _ => Err(format!(
                "unknown automation kind {value:?}; expected cv or projects"
            )),
        }
    }

    /// The one fixed branch owned by this synchronizer.
    #[must_use]
    pub const fn branch(self) -> &'static str {
        match self {
            Self::Cv => "automation/cv-sync",
            Self::Projects => "automation/project-sync",
        }
    }

    /// The workflow-controlled marker placed in the pull-request body.
    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Cv => "<!-- personal-site-sync:cv -->",
            Self::Projects => "<!-- personal-site-sync:projects -->",
        }
    }

    /// The stable title used when creating or updating the one pull request.
    #[must_use]
    pub const fn pull_request_title(self) -> &'static str {
        match self {
            Self::Cv => "chore(cv): synchronize source release",
            Self::Projects => "chore(projects): synchronize portfolio data",
        }
    }
}

/// Pull-request facts collected from GitHub before enabling native auto-merge.
pub struct PullRequestEvidence<'a> {
    pub head_branch: &'a str,
    pub base_branch: &'a str,
    pub head_repository: &'a str,
    pub repository: &'a str,
    pub author: &'a str,
    pub trusted_author: &'a str,
    pub marker_present: bool,
}

/// Returns whether the exact generated PR may receive GitHub native auto-merge.
///
/// Required CI and branch-protection checks are deliberately left to GitHub's
/// native auto-merge state machine, which does not merge until they pass.
#[must_use]
pub fn is_auto_merge_eligible(kind: AutomationKind, evidence: &PullRequestEvidence<'_>) -> bool {
    evidence.head_branch == kind.branch()
        && evidence.base_branch == "main"
        && evidence.head_repository == evidence.repository
        && evidence.author == evidence.trusted_author
        && evidence.marker_present
}
