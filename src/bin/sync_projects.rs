use std::{env, path::PathBuf, process::ExitCode};

use personal_site::project_sync::{
    GitHubProjectSource, ProjectDataStore, SyncOutcome, load_config, synchronize,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("project synchronization failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = parse_root()?;
    let config = load_config(&root)?;
    let token = env::var("PORTFOLIO_GITHUB_TOKEN").ok();
    let source = GitHubProjectSource::new(&config.owner, &config.metadata_path, token.as_deref());
    let store = ProjectDataStore::new(&root);
    match synchronize(&source, &store, &config)? {
        SyncOutcome::Updated {
            projects,
            selection,
        } => println!("updated {projects} projects selected via {selection:?}"),
        SyncOutcome::Unchanged {
            projects,
            selection,
        } => println!("{projects} projects selected via {selection:?}; data is unchanged"),
    }
    Ok(())
}

fn parse_root() -> Result<PathBuf, String> {
    let mut arguments = env::args().skip(1);
    let mut root = PathBuf::from(".");
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                root = PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "--root requires a path".to_owned())?,
                );
            }
            _ => return Err(format!("unknown argument {argument}")),
        }
    }
    Ok(root)
}
