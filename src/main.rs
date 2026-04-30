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

// ── shared types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ConvertRequest {
    url: String,
    scope: Option<String>,
}

#[derive(Serialize)]
struct ConvertResponse {
    markdown: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

type ApiError = (StatusCode, ResponseJson<ErrorResponse>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse { error: msg.into() }))
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_GATEWAY, ResponseJson(ErrorResponse { error: msg.into() }))
}

// ── GitHub URL parser ────────────────────────────────────────────────────────

enum GithubLink<'a> {
    Repo    { owner: &'a str, repo: &'a str },
    Issue   { owner: &'a str, repo: &'a str, number: &'a str },
    Pull    { owner: &'a str, repo: &'a str, number: &'a str },
    Commit  { owner: &'a str, repo: &'a str, sha: &'a str },
    Blob    { owner: &'a str, repo: &'a str, branch: &'a str, path: String },
    Tree    { owner: &'a str, repo: &'a str, branch: &'a str, sub: String },
    Release { owner: &'a str, repo: &'a str, tag: &'a str },
    Other   { owner: &'a str, repo: &'a str },
}

fn parse_github_url<'a>(parts: &'a [&'a str]) -> Result<GithubLink<'a>, String> {
    if parts.len() < 2 {
        return Err("Could not parse owner/repo from URL".to_string());
    }
    let owner = parts[0];
    let repo  = parts[1];

    let link = match parts.get(2) {
        None => GithubLink::Repo { owner, repo },
        Some(&"issues")   if parts.len() >= 4 => GithubLink::Issue   { owner, repo, number: parts[3] },
        Some(&"pull")     if parts.len() >= 4 => GithubLink::Pull    { owner, repo, number: parts[3] },
        Some(&"commit")   if parts.len() >= 4 => GithubLink::Commit  { owner, repo, sha: parts[3] },
        Some(&"blob")     if parts.len() >= 5 => GithubLink::Blob    { owner, repo, branch: parts[3], path: parts[4..].join("/") },
        Some(&"tree")     if parts.len() >= 4 => GithubLink::Tree    {
            owner, repo, branch: parts[3],
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
        GithubLink::Repo    { owner, repo }               => format!("https://api.github.com/repos/{owner}/{repo}"),
        GithubLink::Issue   { owner, repo, number }       => format!("https://api.github.com/repos/{owner}/{repo}/issues/{number}"),
        GithubLink::Pull    { owner, repo, number }       => format!("https://api.github.com/repos/{owner}/{repo}/pulls/{number}"),
        GithubLink::Commit  { owner, repo, sha }          => format!("https://api.github.com/repos/{owner}/{repo}/commits/{sha}"),
        GithubLink::Blob    { owner, repo, branch, path } => format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={branch}"),
        GithubLink::Release { owner, repo, tag }          => format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}"),
        GithubLink::Tree    { owner, repo, .. }
        | GithubLink::Other { owner, repo }               => format!("https://api.github.com/repos/{owner}/{repo}"),
    }
}

// ── GitHub HTTP helper ───────────────────────────────────────────────────────

async fn github_get(url: &str) -> Result<Value, String> {
    let mut req = Client::new()
        .get(url)
        .header("User-Agent", "gh2md/0.1")
        .header("Accept", "application/vnd.github+json");

    if let Ok(token) = env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    req.send().await.map_err(|e| e.to_string())?
       .json().await.map_err(|e| e.to_string())
}

// ── blob decode helper ───────────────────────────────────────────────────────

fn blob_to_code_block(name: &str, data: &Value) -> String {
    let ext     = name.rsplit('.').next().unwrap_or("");
    let raw     = data["content"].as_str().unwrap_or("");
    let cleaned: String = raw.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    let content = STANDARD
        .decode(cleaned.as_bytes())
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_default();
    format!("```{ext}\n{content}```")
}

// ── /convert ─────────────────────────────────────────────────────────────────

fn build_markdown(link: &GithubLink, data: &Value, original_url: &str) -> String {
    let s = |key: &str| data[key].as_str().unwrap_or("").to_string();

    match link {
        GithubLink::Repo { owner, repo } => {
            let desc  = s("description");
            let stars = data["stargazers_count"].as_u64().unwrap_or(0);
            let lang  = s("language");
            let label = if desc.is_empty() { format!("{owner}/{repo}") }
                        else               { format!("{owner}/{repo} — {desc}") };
            let mut md = format!("[{label}]({original_url})");
            if !lang.is_empty() || stars > 0 { md.push_str(&format!(" `{lang}` ⭐{stars}")); }
            md
        }
        GithubLink::Issue { owner, repo, number } => {
            format!("[{owner}/{repo}#{number}: {}]({original_url}) `{}`", s("title"), s("state"))
        }
        GithubLink::Pull { owner, repo, number } => {
            format!("[{owner}/{repo} PR#{number}: {}]({original_url}) `{}`", s("title"), s("state"))
        }
        GithubLink::Commit { owner, repo, sha } => {
            let short = &sha[..7.min(sha.len())];
            let msg   = data["commit"]["message"].as_str().unwrap_or("").lines().next().unwrap_or("");
            format!("[{owner}/{repo}@{short}: {msg}]({original_url})")
        }
        GithubLink::Blob { path, .. } => {
            let name = path.rsplit('/').next().unwrap_or(path.as_str());
            blob_to_code_block(name, data)
        }
        GithubLink::Release { owner, repo, tag } => {
            let name  = s("name");
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
) -> Result<ResponseJson<ConvertResponse>, ApiError> {
    let url = payload.url.trim().trim_end_matches('/');

    if !url.contains("github.com") {
        return Err(bad_request("Not a GitHub URL"));
    }

    let path  = url.split("github.com/").nth(1).ok_or_else(|| bad_request("Invalid GitHub URL"))?;
    let parts: Vec<&str> = path.split('/').collect();
    let link  = parse_github_url(&parts).map_err(bad_request)?;

    // scope: "current-dir" — fetch every file in the same directory
    if payload.scope.as_deref() == Some("current-dir") {
        let GithubLink::Blob { owner, repo, branch, path: file_path } = &link else {
            return Err(bad_request("scope=current-dir requires a file (blob) URL"));
        };

        let dir = file_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        let dir_api = format!(
            "https://api.github.com/repos/{owner}/{repo}/contents/{dir}?ref={branch}"
        );

        let listing  = github_get(&dir_api).await.map_err(bad_gateway)?;
        let entries  = listing.as_array()
            .ok_or_else(|| bad_request("Directory listing was not an array"))?;

        let ext_rank = |name: &str| -> u8 {
            match name.rsplit('.').next().unwrap_or("") {
                "ts"   => 0,
                "html" => 1,
                "css"  => 2,
                "scss" => 3,
                _      => 4,
            }
        };

        let mut files: Vec<&Value> = entries.iter()
            .filter(|e| e["type"].as_str() == Some("file"))
            .collect();
        files.sort_by_key(|e| ext_rank(e["name"].as_str().unwrap_or("")));

        let mut blocks: Vec<String> = Vec::new();
        for entry in files {
            let name      = entry["name"].as_str().unwrap_or("");
            let file_url  = entry["url"].as_str().unwrap_or("");
            let file_data = github_get(file_url).await.map_err(bad_gateway)?;
            blocks.push(format!("### {name}\n\n{}", blob_to_code_block(name, &file_data)));
        }

        return Ok(ResponseJson(ConvertResponse { markdown: blocks.join("\n\n") }));
    }

    // default: single item
    let data     = github_get(&api_url(&link)).await.map_err(bad_gateway)?;
    let markdown = build_markdown(&link, &data, url);
    Ok(ResponseJson(ConvertResponse { markdown }))
}

// ── main ─────────────────────────────────────────────────────────────────────

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
    println!("  POST /convert  {{\"url\": \"...\", \"scope\": \"current-dir\"}}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
