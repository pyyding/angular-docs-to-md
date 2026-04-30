use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::post,
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[derive(Deserialize)]
struct ConvertRequest {
    url: String,
}

#[derive(Serialize)]
struct ConvertResponse {
    markdown: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Parsed representation of a GitHub URL.
enum GithubLink<'a> {
    Repo { owner: &'a str, repo: &'a str },
    Issue { owner: &'a str, repo: &'a str, number: &'a str },
    Pull { owner: &'a str, repo: &'a str, number: &'a str },
    Commit { owner: &'a str, repo: &'a str, sha: &'a str },
    Blob { owner: &'a str, repo: &'a str, branch: &'a str, path: String },
    Tree { owner: &'a str, repo: &'a str, branch: &'a str, sub: String },
    Release { owner: &'a str, repo: &'a str, tag: &'a str },
    Other { owner: &'a str, repo: &'a str },
}

fn parse_github_url<'a>(parts: &'a [&'a str]) -> Result<GithubLink<'a>, String> {
    if parts.len() < 2 {
        return Err("Could not parse owner/repo from URL".to_string());
    }
    let owner = parts[0];
    let repo = parts[1];

    let link = match parts.get(2) {
        None => GithubLink::Repo { owner, repo },
        Some(&"issues") if parts.len() >= 4 => GithubLink::Issue { owner, repo, number: parts[3] },
        Some(&"pull") if parts.len() >= 4 => GithubLink::Pull { owner, repo, number: parts[3] },
        Some(&"commit") if parts.len() >= 4 => GithubLink::Commit { owner, repo, sha: parts[3] },
        Some(&"blob") if parts.len() >= 5 => GithubLink::Blob {
            owner,
            repo,
            branch: parts[3],
            path: parts[4..].join("/"),
        },
        Some(&"tree") if parts.len() >= 4 => GithubLink::Tree {
            owner,
            repo,
            branch: parts[3],
            sub: if parts.len() > 4 { format!("/{}", parts[4..].join("/")) } else { String::new() },
        },
        Some(&"releases") if parts.get(3) == Some(&"tag") && parts.len() >= 5 => {
            GithubLink::Release { owner, repo, tag: parts[4] }
        }
        _ => GithubLink::Other { owner, repo },
    };
    Ok(link)
}

fn api_url(link: &GithubLink) -> String {
    match link {
        GithubLink::Repo { owner, repo } => {
            format!("https://api.github.com/repos/{owner}/{repo}")
        }
        GithubLink::Issue { owner, repo, number } => {
            format!("https://api.github.com/repos/{owner}/{repo}/issues/{number}")
        }
        GithubLink::Pull { owner, repo, number } => {
            format!("https://api.github.com/repos/{owner}/{repo}/pulls/{number}")
        }
        GithubLink::Commit { owner, repo, sha } => {
            format!("https://api.github.com/repos/{owner}/{repo}/commits/{sha}")
        }
        GithubLink::Blob { owner, repo, branch, path } => {
            format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={branch}")
        }
        GithubLink::Release { owner, repo, tag } => {
            format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}")
        }
        // Tree and Other don't have a clean single API endpoint — fall back to repo
        GithubLink::Tree { owner, repo, .. } | GithubLink::Other { owner, repo } => {
            format!("https://api.github.com/repos/{owner}/{repo}")
        }
    }
}

fn build_markdown(link: &GithubLink, data: &Value, original_url: &str) -> String {
    let s = |key: &str| data[key].as_str().unwrap_or("").to_string();

    match link {
        GithubLink::Repo { owner, repo } => {
            let desc = s("description");
            let stars = data["stargazers_count"].as_u64().unwrap_or(0);
            let lang = s("language");
            let label = if desc.is_empty() {
                format!("{owner}/{repo}")
            } else {
                format!("{owner}/{repo} — {desc}")
            };
            let mut md = format!("[{label}]({original_url})");
            if !lang.is_empty() || stars > 0 {
                md.push_str(&format!(" `{lang}` ⭐{stars}"));
            }
            md
        }
        GithubLink::Issue { owner, repo, number } => {
            let title = s("title");
            let state = s("state");
            format!("[{owner}/{repo}#{number}: {title}]({original_url}) `{state}`")
        }
        GithubLink::Pull { owner, repo, number } => {
            let title = s("title");
            let state = s("state");
            format!("[{owner}/{repo} PR#{number}: {title}]({original_url}) `{state}`")
        }
        GithubLink::Commit { owner, repo, sha } => {
            let short = &sha[..7.min(sha.len())];
            let msg = data["commit"]["message"]
                .as_str()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            format!("[{owner}/{repo}@{short}: {msg}]({original_url})")
        }
        GithubLink::Blob { path, .. } => {
            let ext = path.rsplit('.').next().unwrap_or("");
            let raw = data["content"].as_str().unwrap_or("");
            // GitHub wraps base64 in newlines — strip them before decoding
            let cleaned: String = raw.chars().filter(|c| *c != '\n' && *c != '\r').collect();
            let content = STANDARD
                .decode(cleaned.as_bytes())
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
                .unwrap_or_default();
            format!("```{ext}\n{content}```")
        }
        GithubLink::Release { owner, repo, tag } => {
            let name = s("name");
            let label = if name.is_empty() { tag.to_string() } else { name };
            format!("[{owner}/{repo} {label}]({original_url})")
        }
        GithubLink::Tree { owner, repo, branch, sub } => {
            format!("[{owner}/{repo}{sub}]({original_url}) `{branch}`")
        }
        GithubLink::Other { owner, repo } => {
            format!("[{owner}/{repo}]({original_url})")
        }
    }
}

async fn convert(
    Json(payload): Json<ConvertRequest>,
) -> Result<ResponseJson<ConvertResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let url = payload.url.trim().trim_end_matches('/');

    if !url.contains("github.com") {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse { error: "Not a GitHub URL".into() }),
        ));
    }

    let path = url
        .split("github.com/")
        .nth(1)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse { error: "Invalid GitHub URL".into() })))?;

    let parts: Vec<&str> = path.split('/').collect();

    let link = parse_github_url(&parts).map_err(|e| {
        (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse { error: e }))
    })?;

    // Call GitHub API
    let api = api_url(&link);

    let mut req = Client::new()
        .get(&api)
        .header("User-Agent", "gh2md/0.1")
        .header("Accept", "application/vnd.github+json");

    if let Ok(token) = env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let data: Value = req
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, ResponseJson(ErrorResponse { error: e.to_string() })))?
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, ResponseJson(ErrorResponse { error: e.to_string() })))?;

    let markdown = build_markdown(&link, &data, url);

    Ok(ResponseJson(ConvertResponse { markdown }))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/convert", post(convert));

    let addr = "0.0.0.0:3000";
    println!("gh2md API listening on http://{addr}");
    if env::var("GITHUB_TOKEN").is_ok() {
        println!("  GitHub token: ✓ (5000 req/hour)");
    } else {
        println!("  GitHub token: ✗ (60 req/hour — set GITHUB_TOKEN to increase)");
    }
    println!();
    println!("  POST /convert  {{\"url\": \"https://github.com/...\"}}");;

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
