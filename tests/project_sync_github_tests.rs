use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    thread::{self, JoinHandle},
};

use personal_site::project_sync::{GitHubProjectSource, ProjectSource, ProjectSyncError};

struct Response {
    body: &'static str,
}

fn graphql_server(responses: Vec<Response>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            assert_eq!(request_line, "POST /graphql HTTP/1.1\r\n");
            let mut content_length = 0;
            let mut authenticated = false;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                if header == "\r\n" || header.is_empty() {
                    break;
                }
                let lowercase = header.to_ascii_lowercase();
                authenticated |= lowercase == "authorization: bearer test-token\r\n";
                if let Some(length) = lowercase.strip_prefix("content-length: ") {
                    content_length = length.trim().parse().unwrap();
                }
            }
            assert!(authenticated);
            let mut request_body = vec![0; content_length];
            reader.read_exact(&mut request_body).unwrap();
            let request_body = String::from_utf8(request_body).unwrap();
            assert!(request_body.contains("PortfolioList"));

            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response.body.len(),
                response.body
            )
            .unwrap();
        }
    });
    (address, handle)
}

#[test]
fn github_adapter_reads_and_paginates_the_named_portfolio_list() {
    let (base_url, server) = graphql_server(vec![
        Response {
            body: r#"{
                "data":{"viewer":{"lists":{
                    "nodes":[{"id":"list-id","name":"portfolio","items":{
                        "nodes":[{"nameWithOwner":"osdesa/personal-site"}],
                        "pageInfo":{"hasNextPage":true,"endCursor":"item-cursor"}
                    }}],
                    "pageInfo":{"hasNextPage":false,"endCursor":"list-cursor"}
                }}}
            }"#,
        },
        Response {
            body: r#"{
                "data":{"node":{"items":{
                    "nodes":[{"nameWithOwner":"osdesa/Blocky"}],
                    "pageInfo":{"hasNextPage":false,"endCursor":"done"}
                }}}
            }"#,
        },
    ]);
    let source = GitHubProjectSource::with_base_url(
        "osdesa",
        ".github/portfolio.toml",
        base_url,
        Some("test-token"),
    );

    let repositories = source
        .named_list_repositories("portfolio")
        .unwrap()
        .unwrap();

    server.join().unwrap();
    assert_eq!(repositories, ["osdesa/personal-site", "osdesa/Blocky"]);
}

#[test]
fn graphql_errors_are_reported_instead_of_switching_selection_sources() {
    let (base_url, server) = graphql_server(vec![Response {
        body: r#"{"errors":[{"message":"starring permission required"}]}"#,
    }]);
    let source = GitHubProjectSource::with_base_url(
        "osdesa",
        ".github/portfolio.toml",
        base_url,
        Some("test-token"),
    );

    let result = source.named_list_repositories("portfolio");

    server.join().unwrap();
    assert!(
        matches!(result, Err(ProjectSyncError::Remote(message)) if message.contains("starring permission required"))
    );
}
