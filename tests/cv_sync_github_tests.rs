use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread::{self, JoinHandle},
};

use personal_site::cv_sync::{
    CvBundleStore, CvSyncError, GitHubCvSource, SyncOutcome, synchronize,
};
use tempfile::TempDir;

const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PDF: &[u8] = include_bytes!("../public/cv/Hayden-Farrell-CV.pdf");
const TEX: &[u8] = b"\\documentclass{article}\n\\begin{document}\nCV\n\\end{document}\n";

struct Response {
    expected_target: &'static str,
    status: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

fn local_server(responses: Vec<Response>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            assert!(
                request_line.starts_with(&format!("GET {} ", response.expected_target)),
                "unexpected request: {request_line}"
            );

            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                if header == "\r\n" || header.is_empty() {
                    break;
                }
            }

            write!(
                stream,
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response.status,
                response.content_type,
                response.body.len()
            )
            .unwrap();
            stream.write_all(&response.body).unwrap();
        }
    });
    (address, handle)
}

#[test]
fn github_adapter_lists_tags_and_downloads_both_files_by_commit_sha() {
    let tags = format!(
        r#"[
            {{"name":"notes","commit":{{"sha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}}},
            {{"name":"v1.0.0","commit":{{"sha":"{SHA}"}}}}
        ]"#
    );
    let (base_url, server) = local_server(vec![
        Response {
            expected_target: "/repos/osdesa/cv/tags?per_page=100&page=1",
            status: "200 OK",
            content_type: "application/json",
            body: tags.into_bytes(),
        },
        Response {
            expected_target: "/osdesa/cv/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Hayden-Farrell-CV.tex",
            status: "200 OK",
            content_type: "text/plain",
            body: TEX.to_vec(),
        },
        Response {
            expected_target: "/osdesa/cv/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Hayden-Farrell-CV.pdf",
            status: "200 OK",
            content_type: "application/pdf",
            body: PDF.to_vec(),
        },
    ]);
    let source =
        GitHubCvSource::with_base_urls("osdesa", "cv", &base_url, &base_url, Some("test-token"))
            .unwrap();
    let root = TempDir::new().unwrap();

    let outcome = synchronize(&source, &CvBundleStore::new(root.path())).unwrap();

    server.join().unwrap();
    assert_eq!(
        outcome,
        SyncOutcome::Updated {
            tag: "v1.0.0".to_owned(),
            commit_sha: SHA.to_owned(),
        }
    );
    assert_eq!(
        fs::read(root.path().join("public/cv/Hayden-Farrell-CV.tex")).unwrap(),
        TEX
    );
}

#[test]
fn github_http_failure_is_reported_before_any_local_files_are_created() {
    let (base_url, server) = local_server(vec![Response {
        expected_target: "/repos/osdesa/cv/tags?per_page=100&page=1",
        status: "503 Service Unavailable",
        content_type: "application/json",
        body: br#"{"message":"try later"}"#.to_vec(),
    }]);
    let source =
        GitHubCvSource::with_base_urls("osdesa", "cv", &base_url, &base_url, None).unwrap();
    let root = TempDir::new().unwrap();

    let result = synchronize(&source, &CvBundleStore::new(root.path()));

    server.join().unwrap();
    assert!(matches!(result, Err(CvSyncError::Remote(_))));
    assert!(!root.path().join("public/cv").exists());
}
