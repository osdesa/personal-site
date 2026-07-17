use std::{env, path::PathBuf, process::ExitCode};

use personal_site::cv_sync::{CvBundleStore, GitHubCvSource, synchronize};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("CV synchronization failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let Some(root) = parse_root(env::args().skip(1))? else {
        return Ok(());
    };
    let token = env::var("GITHUB_TOKEN").ok();
    let source =
        GitHubCvSource::new("osdesa", "cv", token.as_deref()).map_err(|error| error.to_string())?;
    let outcome =
        synchronize(&source, &CvBundleStore::new(root)).map_err(|error| error.to_string())?;
    println!("{outcome}");
    Ok(())
}

fn parse_root(mut arguments: impl Iterator<Item = String>) -> Result<Option<PathBuf>, String> {
    let mut root = PathBuf::from(".");
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                root = arguments
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| "--root requires a repository path".to_owned())?;
            }
            "--help" | "-h" => {
                println!(
                    "Synchronize the highest osdesa/cv semantic tag.\n\nUsage: sync-cv [--root <repository-path>]"
                );
                return Ok(None);
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    Ok(Some(root))
}
