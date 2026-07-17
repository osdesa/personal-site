use personal_site::cv_sync::{
    CvManifest, RemoteTag, parse_semantic_tag, select_highest_semantic_tag, validate_commit_sha,
    validate_pdf, validate_tex,
};

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const VALID_TEX: &[u8] = b"\\documentclass{article}\n\\begin{document}\nCV\n\\end{document}\n";
const CHECKED_IN_PDF: &[u8] = include_bytes!("../public/cv/Hayden-Farrell-CV.pdf");

fn tag(name: &str, commit_sha: &str) -> RemoteTag {
    RemoteTag {
        name: name.to_owned(),
        commit_sha: commit_sha.to_owned(),
    }
}

#[test]
fn semantic_tag_parser_accepts_conventional_prefixes() {
    assert_eq!(parse_semantic_tag("v1.2.3").unwrap().to_string(), "1.2.3");
    assert_eq!(parse_semantic_tag("V2.0.0-rc.1").unwrap().major, 2);
    assert!(parse_semantic_tag("release-1.2.3").is_none());
}

#[test]
fn highest_tag_uses_semantic_not_lexical_order() {
    let selected = select_highest_semantic_tag(&[
        tag("not-a-version", SHA_A),
        tag("v1.9.0", SHA_A),
        tag("v1.10.0-rc.1", SHA_A),
        tag("v1.10.0", SHA_B),
    ])
    .unwrap();

    assert_eq!(selected, tag("v1.10.0", SHA_B));
}

#[test]
fn equal_semantic_precedence_has_a_deterministic_tie_breaker() {
    let selected =
        select_highest_semantic_tag(&[tag("v1.0.0+build.1", SHA_A), tag("v1.0.0+build.2", SHA_B)])
            .unwrap();

    assert_eq!(selected.name, "v1.0.0+build.2");
}

#[test]
fn commit_sha_requires_a_full_lowercase_git_sha() {
    assert!(validate_commit_sha(SHA_A).is_ok());
    assert!(validate_commit_sha("abc123").is_err());
    assert!(validate_commit_sha("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_err());
    assert!(validate_commit_sha("gggggggggggggggggggggggggggggggggggggggg").is_err());
}

#[test]
fn tex_validation_rejects_truncated_and_non_utf8_documents() {
    assert!(validate_tex(VALID_TEX).is_ok());
    assert!(validate_tex(b"<html>Not Found</html>").is_err());
    assert!(validate_tex(b"\\begin{document}\n\\documentclass{x}\n\\end{document}").is_err());
    assert!(validate_tex(&[0xff, 0xfe]).is_err());
}

#[test]
fn pdf_validation_parses_the_document_and_rejects_wrappers_or_truncation() {
    assert!(validate_pdf(CHECKED_IN_PDF).is_ok());
    assert!(validate_pdf(b"<html>Not Found</html>").is_err());
    assert!(validate_pdf(b"%PDF-1.7\nnot a PDF\n%%EOF\n").is_err());

    let without_eof = &CHECKED_IN_PDF[..CHECKED_IN_PDF.len() - 16];
    assert!(validate_pdf(without_eof).is_err());
}

#[test]
fn manifest_rejects_unknown_or_untrusted_metadata() {
    let json = format!(
        r#"{{
            "schema_version": 1,
            "repository": "someone/else",
            "tag": "v1.0.0",
            "commit_sha": "{SHA_A}",
            "source": {{"filename":"Hayden-Farrell-CV.tex","bytes":1,"sha256":"{}"}},
            "pdf": {{"filename":"Hayden-Farrell-CV.pdf","bytes":1,"sha256":"{}"}}
        }}"#,
        "a".repeat(64),
        "b".repeat(64)
    );
    let manifest: CvManifest = serde_json::from_str(&json).unwrap();

    assert!(manifest.validate_metadata().is_err());

    let with_unknown_field = json.replace(
        "\"schema_version\": 1,",
        "\"schema_version\": 1, \"unexpected\": true,",
    );
    assert!(serde_json::from_str::<CvManifest>(&with_unknown_field).is_err());
}
