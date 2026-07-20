use personal_site::automation::{AutomationKind, PullRequestEvidence, is_auto_merge_eligible};

fn evidence(kind: AutomationKind) -> PullRequestEvidence<'static> {
    PullRequestEvidence {
        head_branch: kind.branch(),
        base_branch: "main",
        head_repository: "osdesa/personal-site",
        repository: "osdesa/personal-site",
        author: "portfolio-sync-bot",
        trusted_author: "portfolio-sync-bot",
        marker_present: true,
    }
}

#[test]
fn fixed_automation_branches_and_pr_metadata_are_stable() {
    assert_eq!(AutomationKind::Cv.branch(), "automation/cv-sync");
    assert_eq!(AutomationKind::Projects.branch(), "automation/project-sync");
    assert_eq!(
        AutomationKind::Cv.marker(),
        "<!-- personal-site-sync:cv -->"
    );
    assert_eq!(
        AutomationKind::Projects.marker(),
        "<!-- personal-site-sync:projects -->"
    );
    assert_eq!(
        AutomationKind::Cv.pull_request_title(),
        "chore(cv): synchronize source release"
    );
    assert_eq!(
        AutomationKind::Projects.pull_request_title(),
        "chore(projects): synchronize portfolio data"
    );
}

#[test]
fn only_trusted_cv_and_project_automation_prs_are_eligible() {
    assert!(is_auto_merge_eligible(
        AutomationKind::Cv,
        &evidence(AutomationKind::Cv)
    ));
    assert!(is_auto_merge_eligible(
        AutomationKind::Projects,
        &evidence(AutomationKind::Projects)
    ));
}

#[test]
fn unrelated_branches_forks_wrong_base_and_untrusted_markers_are_rejected() {
    let mut unrelated_branch = evidence(AutomationKind::Cv);
    unrelated_branch.head_branch = "automation/cv-sync-extra";
    assert!(!is_auto_merge_eligible(
        AutomationKind::Cv,
        &unrelated_branch
    ));

    let mut fork = evidence(AutomationKind::Cv);
    fork.head_repository = "contributor/personal-site";
    assert!(!is_auto_merge_eligible(AutomationKind::Cv, &fork));

    let mut wrong_base = evidence(AutomationKind::Cv);
    wrong_base.base_branch = "release";
    assert!(!is_auto_merge_eligible(AutomationKind::Cv, &wrong_base));

    let mut wrong_author = evidence(AutomationKind::Cv);
    wrong_author.author = "contributor";
    assert!(!is_auto_merge_eligible(AutomationKind::Cv, &wrong_author));

    let mut missing_marker = evidence(AutomationKind::Cv);
    missing_marker.marker_present = false;
    assert!(!is_auto_merge_eligible(AutomationKind::Cv, &missing_marker));
}
