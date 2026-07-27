use std::{
    collections::{HashMap, HashSet},
    fs,
};

use personal_site::project_sync::{
    GENERATED_PROJECTS_PATH, NormalizedProject, PortfolioMetadata, ProjectDataStore, ProjectSource,
    ProjectSyncConfig, ProjectSyncError, RemoteRepository, SelectionMethod, SyncOutcome,
    generate_projects_module, normalize_projects, parse_portfolio_metadata, synchronize,
};

const TEST_OWNER: &str = "example-owner";
const SELECTED_REPOSITORY: &str = "example-owner/selected-project";
const FIRST_FALLBACK: &str = "example-owner/fallback-one";
const SECOND_FALLBACK: &str = "example-owner/fallback-two";

fn repository(name: &str, created: &str) -> RemoteRepository {
    RemoteRepository {
        name: name.split('/').next_back().unwrap_or(name).to_owned(),
        full_name: name.to_owned(),
        description: Some(format!("Description for {name}")),
        private: false,
        html_url: format!("https://github.com/{name}"),
        homepage: None,
        created_at: format!("{created}T12:00:00Z"),
        language: Some("Rust".to_owned()),
        topics: vec!["portfolio".to_owned(), "leptos".to_owned()],
        archived: false,
        fork: false,
    }
}

fn config() -> ProjectSyncConfig {
    ProjectSyncConfig {
        owner: TEST_OWNER.to_owned(),
        list: "portfolio".to_owned(),
        topic: "portfolio".to_owned(),
        metadata_path: ".github/portfolio.toml".to_owned(),
        limit: 4,
        fallback_repositories: vec![FIRST_FALLBACK.to_owned(), SECOND_FALLBACK.to_owned()],
    }
}

struct MockSource {
    named_list: Result<Option<Vec<String>>, String>,
    accessible: Result<Vec<RemoteRepository>, String>,
    repositories: HashMap<String, RemoteRepository>,
    metadata: HashMap<String, Result<Option<String>, String>>,
    thumbnails: HashMap<String, Result<Option<Vec<u8>>, String>>,
}

impl ProjectSource for MockSource {
    fn named_list_repositories(
        &self,
        _list_name: &str,
    ) -> Result<Option<Vec<String>>, ProjectSyncError> {
        self.named_list.clone().map_err(ProjectSyncError::Remote)
    }

    fn accessible_repositories(&self) -> Result<Vec<RemoteRepository>, ProjectSyncError> {
        self.accessible.clone().map_err(ProjectSyncError::Remote)
    }

    fn repository(&self, full_name: &str) -> Result<RemoteRepository, ProjectSyncError> {
        self.repositories.get(full_name).cloned().ok_or_else(|| {
            ProjectSyncError::Remote(format!("missing repository fixture {full_name}"))
        })
    }

    fn portfolio_metadata(&self, full_name: &str) -> Result<Option<String>, ProjectSyncError> {
        self.metadata
            .get(full_name)
            .cloned()
            .unwrap_or(Ok(None))
            .map_err(ProjectSyncError::Remote)
    }

    fn project_thumbnail(&self, full_name: &str) -> Result<Option<Vec<u8>>, ProjectSyncError> {
        self.thumbnails
            .get(full_name)
            .cloned()
            .unwrap_or(Ok(None))
            .map_err(ProjectSyncError::Remote)
    }
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes
}

#[test]
fn named_list_is_the_primary_selection_source() {
    let selected = repository(SELECTED_REPOSITORY, "2026-01-01");
    let source = MockSource {
        named_list: Ok(Some(vec![SELECTED_REPOSITORY.to_owned()])),
        accessible: Ok(vec![selected]),
        repositories: HashMap::from([(
            SELECTED_REPOSITORY.to_owned(),
            repository(SELECTED_REPOSITORY, "2026-01-01"),
        )]),
        metadata: HashMap::new(),
        thumbnails: HashMap::new(),
    };
    let root = tempfile::tempdir().unwrap();
    let outcome = synchronize(&source, &ProjectDataStore::new(root.path()), &config()).unwrap();
    assert_eq!(
        outcome,
        SyncOutcome::Updated {
            projects: 1,
            selection: SelectionMethod::NamedList
        }
    );
    let generated = fs::read_to_string(root.path().join(GENERATED_PROJECTS_PATH)).unwrap();
    assert!(generated.contains(SELECTED_REPOSITORY));
    assert!(!generated.contains(FIRST_FALLBACK));
}

#[test]
fn topic_then_allowlist_are_ordered_fallbacks() {
    let topic_source = MockSource {
        named_list: Ok(None),
        accessible: Ok(vec![repository(SELECTED_REPOSITORY, "2026-01-01")]),
        repositories: HashMap::new(),
        metadata: HashMap::new(),
        thumbnails: HashMap::new(),
    };
    let root = tempfile::tempdir().unwrap();
    let outcome = synchronize(
        &topic_source,
        &ProjectDataStore::new(root.path()),
        &config(),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        SyncOutcome::Updated {
            selection: SelectionMethod::Topic,
            projects: 1
        }
    ));

    let fallback_repositories = config()
        .fallback_repositories
        .iter()
        .map(|name| (name.clone(), repository(name, "2025-01-01")))
        .collect();
    let fallback_source = MockSource {
        named_list: Ok(Some(Vec::new())),
        accessible: Ok(vec![repository("another-owner/irrelevant", "2026-01-01")]),
        repositories: fallback_repositories,
        metadata: HashMap::new(),
        thumbnails: HashMap::new(),
    };
    let root = tempfile::tempdir().unwrap();
    let outcome = synchronize(
        &fallback_source,
        &ProjectDataStore::new(root.path()),
        &config(),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        SyncOutcome::Updated {
            selection: SelectionMethod::AllowlistFallback,
            projects: 2
        }
    ));
}

#[test]
fn metadata_parses_and_overrides_github_fallbacks() {
    let metadata = parse_portfolio_metadata(
        r#"
title = "Display title"
summary = "Portfolio summary"
date = "2026-05-04"
status = "Active"
technologies = ["Rust", "WebAssembly"]
highlights = ["Deterministic output"]
demo_url = "https://example.com/demo"
show_repository = false
"#,
    )
    .unwrap();
    let projects = normalize_projects(
        vec![(
            repository("example-owner/example-name", "2024-01-02"),
            metadata,
        )],
        4,
    )
    .unwrap();
    let project = &projects[0];
    assert_eq!(project.id, "example-owner-example-name");
    assert_eq!(project.title, "Display title");
    assert_eq!(project.summary, "Portfolio summary");
    assert_eq!(project.portfolio_date.as_deref(), Some("2026-05-04"));
    assert_eq!(project.status.as_deref(), Some("Active"));
    assert_eq!(project.technologies, ["Rust", "WebAssembly"]);
    assert_eq!(project.highlights, ["Deterministic output"]);
    assert_eq!(project.image_url, "/images/project-default.svg");
    assert_eq!(
        project.demo_url.as_deref(),
        Some("https://example.com/demo")
    );
    assert!(project.repository_url.is_none());
}

#[test]
fn imported_project_links_require_complete_credential_free_https_urls() {
    for invalid in [
        "https://",
        "https://user:password@example.com/demo",
        "javascript:alert(1)",
    ] {
        let source = format!("demo_url = {invalid:?}");
        assert!(matches!(
            parse_portfolio_metadata(&source),
            Err(ProjectSyncError::Validation(message))
                if message.contains("absolute HTTPS URL without credentials")
        ));
    }

    let mut invalid_repository = repository("example-owner/example", "2026-01-01");
    invalid_repository.html_url = "https://".to_owned();
    assert!(matches!(
        normalize_projects(
            vec![(invalid_repository, PortfolioMetadata::default())],
            4
        ),
        Err(ProjectSyncError::Validation(message))
            if message.contains("repository URL")
    ));
}

#[test]
fn image_metadata_is_rejected_in_favour_of_repository_thumbnails() {
    let error = parse_portfolio_metadata(r#"image = "/images/project-default.svg""#).unwrap_err();

    assert!(
        matches!(error, ProjectSyncError::Validation(message) if message.contains("unknown field"))
    );
}

#[test]
fn github_metadata_fallbacks_support_public_and_private_repositories() {
    let mut public = repository("example-owner/example-project", "2024-02-03");
    public.description = None;
    public.homepage = Some("https://example.com".to_owned());
    let mut private = repository("example-owner/secret-tool", "2025-02-03");
    private.private = true;
    let projects = normalize_projects(
        vec![
            (public, PortfolioMetadata::default()),
            (private, PortfolioMetadata::default()),
        ],
        4,
    )
    .unwrap();
    let public = projects
        .iter()
        .find(|project| project.repository.ends_with("example-project"))
        .unwrap();
    assert_eq!(public.title, "Example Project");
    assert_eq!(public.summary, "A software project from example-owner.");
    assert_eq!(public.demo_url.as_deref(), Some("https://example.com"));
    assert_eq!(public.technologies, ["portfolio", "leptos", "Rust"]);
    let private = projects.iter().find(|project| project.private).unwrap();
    assert!(private.repository_url.is_none());
}

#[test]
fn sorting_is_deterministic_and_limited_to_four_newest_projects() {
    let mut candidates = (0..6)
        .map(|index| {
            (
                repository(
                    &format!("example-owner/project-{index}"),
                    &format!("202{}-01-01", index % 4 + 2),
                ),
                PortfolioMetadata::default(),
            )
        })
        .collect::<Vec<_>>();
    candidates[0].1.date = Some("2030-01-01".to_owned());
    let projects = normalize_projects(candidates, 4).unwrap();
    let ids = projects
        .iter()
        .map(|project| project.id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(projects.len(), 4);
    assert_eq!(ids.len(), projects.len());
    assert!(ids.iter().all(|id| {
        !id.is_empty()
            && id.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
    }));
    assert_eq!(projects[0].repository, "example-owner/project-0");
    assert!(projects.windows(2).all(|pair| {
        let left = pair[0]
            .portfolio_date
            .as_ref()
            .unwrap_or(&pair[0].created_date);
        let right = pair[1]
            .portfolio_date
            .as_ref()
            .unwrap_or(&pair[1].created_date);
        left >= right
    }));
    assert_eq!(
        generate_projects_module(&projects),
        generate_projects_module(&projects)
    );
}

#[test]
fn archived_and_forked_repositories_require_explicit_metadata_opt_in() {
    let mut archived = repository("example-owner/archived", "2025-01-01");
    archived.archived = true;
    let mut fork = repository("example-owner/fork", "2025-01-02");
    fork.fork = true;
    let included = PortfolioMetadata {
        include_archived: true,
        include_fork: true,
        ..PortfolioMetadata::default()
    };
    let projects = normalize_projects(
        vec![
            (archived.clone(), PortfolioMetadata::default()),
            (fork.clone(), PortfolioMetadata::default()),
            (archived, included.clone()),
            (fork, included),
        ],
        4,
    )
    .unwrap();
    assert_eq!(projects.len(), 2);
}

#[test]
fn failed_synchronization_preserves_previous_generated_data() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join(GENERATED_PROJECTS_PATH);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"previous valid project data\n").unwrap();
    let selected = repository("example-owner/broken", "2026-01-01");
    let source = MockSource {
        named_list: Ok(Some(vec!["example-owner/broken".to_owned()])),
        accessible: Ok(vec![selected]),
        repositories: HashMap::from([(
            "example-owner/broken".to_owned(),
            repository("example-owner/broken", "2026-01-01"),
        )]),
        metadata: HashMap::from([(
            "example-owner/broken".to_owned(),
            Ok(Some("date = 'not-a-date'".to_owned())),
        )]),
        thumbnails: HashMap::new(),
    };

    assert!(synchronize(&source, &ProjectDataStore::new(root.path()), &config()).is_err());
    assert_eq!(
        fs::read_to_string(target).unwrap(),
        "previous valid project data\n"
    );
}

#[test]
fn repository_thumbnail_is_copied_into_the_static_bundle() {
    let source = MockSource {
        named_list: Ok(Some(vec!["example-owner/thumbnail-project".to_owned()])),
        accessible: Ok(Vec::new()),
        repositories: HashMap::from([(
            "example-owner/thumbnail-project".to_owned(),
            repository("example-owner/thumbnail-project", "2026-01-01"),
        )]),
        metadata: HashMap::new(),
        thumbnails: HashMap::from([(
            "example-owner/thumbnail-project".to_owned(),
            Ok(Some(png(608, 272))),
        )]),
    };
    let root = tempfile::tempdir().unwrap();

    synchronize(&source, &ProjectDataStore::new(root.path()), &config()).unwrap();

    let generated = fs::read_to_string(root.path().join(GENERATED_PROJECTS_PATH)).unwrap();
    assert!(generated.contains("/images/projects/example-owner-thumbnail-project.png"));
    assert_eq!(
        fs::read(
            root.path()
                .join("public/images/projects/example-owner-thumbnail-project.png")
        )
        .unwrap(),
        png(608, 272)
    );
}

#[test]
fn invalid_thumbnail_preserves_existing_generated_data() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join(GENERATED_PROJECTS_PATH);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"previous valid project data\n").unwrap();
    let source = MockSource {
        named_list: Ok(Some(vec!["example-owner/broken-thumbnail".to_owned()])),
        accessible: Ok(Vec::new()),
        repositories: HashMap::from([(
            "example-owner/broken-thumbnail".to_owned(),
            repository("example-owner/broken-thumbnail", "2026-01-01"),
        )]),
        metadata: HashMap::new(),
        thumbnails: HashMap::from([(
            "example-owner/broken-thumbnail".to_owned(),
            Ok(Some(b"not a PNG".to_vec())),
        )]),
    };

    assert!(synchronize(&source, &ProjectDataStore::new(root.path()), &config()).is_err());
    assert_eq!(
        fs::read_to_string(target).unwrap(),
        "previous valid project data\n"
    );
}

#[test]
fn generated_output_escapes_untrusted_repository_text() {
    let project = NormalizedProject {
        id: "example".to_owned(),
        repository: "example-owner/example".to_owned(),
        title: "A \"quoted\" title".to_owned(),
        summary: "line one\nline two".to_owned(),
        private: false,
        created_date: "2026-01-01".to_owned(),
        portfolio_date: None,
        status: None,
        technologies: vec!["Rust".to_owned()],
        highlights: Vec::new(),
        image_url: "/images/project-default.svg".to_owned(),
        repository_url: Some("https://example.test/example-owner/example".to_owned()),
        demo_url: None,
    };
    let generated = String::from_utf8(generate_projects_module(&[project])).unwrap();
    assert!(generated.contains(r#"title: "A \"quoted\" title""#));
    assert!(generated.contains(r#"summary: "line one\nline two""#));
}

#[test]
fn generated_single_highlight_matches_rustfmt_line_width() {
    let highlight = "A generated highlight near the formatter boundary stays on a stable line.";
    let project = NormalizedProject {
        id: "example-owner-formatter-case".to_owned(),
        repository: "example-owner/formatter-case".to_owned(),
        title: "Formatter Case".to_owned(),
        summary: "A controlled project used to exercise generated formatting.".to_owned(),
        private: false,
        created_date: "2026-07-24".to_owned(),
        portfolio_date: None,
        status: Some("Active".to_owned()),
        technologies: vec!["Example language".to_owned(), "Example tool".to_owned()],
        highlights: vec![highlight.to_owned()],
        image_url: "/images/project-default.svg".to_owned(),
        repository_url: Some("https://example.test/example-owner/formatter-case".to_owned()),
        demo_url: None,
    };

    let generated = String::from_utf8(generate_projects_module(&[project])).unwrap();

    assert!(generated.contains(&format!("        highlights: &[{highlight:?}],")));
}
