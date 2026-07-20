use std::{env, process::ExitCode};

use personal_site::automation::{AutomationKind, PullRequestEvidence, is_auto_merge_eligible};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("auto-merge eligibility verification failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = Arguments::parse(env::args().skip(1))?;
    let kind = AutomationKind::parse(&arguments.kind)?;
    let evidence = PullRequestEvidence {
        head_branch: &arguments.head_branch,
        base_branch: &arguments.base_branch,
        head_repository: &arguments.head_repository,
        repository: &arguments.repository,
        author: &arguments.author,
        trusted_author: &arguments.trusted_author,
        marker_present: arguments.marker_present,
    };
    if !is_auto_merge_eligible(kind, &evidence) {
        return Err(format!(
            "PR is not the trusted {} automation PR (expected branch {}, base main, same-repository head, author {}, and body marker {})",
            arguments.kind,
            kind.branch(),
            arguments.trusted_author,
            kind.marker(),
        ));
    }
    println!(
        "verified trusted {} synchronization pull request",
        arguments.kind
    );
    Ok(())
}

#[derive(Default)]
struct Arguments {
    kind: String,
    head_branch: String,
    base_branch: String,
    head_repository: String,
    repository: String,
    author: String,
    trusted_author: String,
    marker_present: bool,
}

impl Arguments {
    fn parse(mut values: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut arguments = Self::default();
        while let Some(flag) = values.next() {
            if flag == "--marker-present" {
                arguments.marker_present = true;
                continue;
            }
            let target = match flag.as_str() {
                "--kind" => &mut arguments.kind,
                "--head-branch" => &mut arguments.head_branch,
                "--base-branch" => &mut arguments.base_branch,
                "--head-repository" => &mut arguments.head_repository,
                "--repository" => &mut arguments.repository,
                "--author" => &mut arguments.author,
                "--trusted-author" => &mut arguments.trusted_author,
                "--help" | "-h" => return Err(usage()),
                _ => return Err(format!("unknown argument {flag}\n\n{}", usage())),
            };
            *target = next_value(&mut values, &flag)?;
        }
        for (name, value) in [
            ("--kind", &arguments.kind),
            ("--head-branch", &arguments.head_branch),
            ("--base-branch", &arguments.base_branch),
            ("--head-repository", &arguments.head_repository),
            ("--repository", &arguments.repository),
            ("--author", &arguments.author),
            ("--trusted-author", &arguments.trusted_author),
        ] {
            if value.is_empty() {
                return Err(format!("{name} is required\n\n{}", usage()));
            }
        }
        Ok(arguments)
    }
}

fn next_value(values: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    values
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} requires a non-empty value\n\n{}", usage()))
}

fn usage() -> String {
    "Usage: verify-auto-merge --kind <cv|projects> --head-branch <branch> --base-branch <branch> --head-repository <owner/repo> --repository <owner/repo> --author <login> --trusted-author <login> --marker-present".to_owned()
}
