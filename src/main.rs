use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

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

fn github_url_to_markdown(url: &str) -> Result<String, String> {
    let url = url.trim();

    // Basic GitHub URL patterns:
    // https://github.com/{owner}/{repo}
    // https://github.com/{owner}/{repo}/blob/{branch}/{path}
    // https://github.com/{owner}/{repo}/tree/{branch}/{path}
    // https://github.com/{owner}/{repo}/issues/{number}
    // https://github.com/{owner}/{repo}/pull/{number}
    // https://github.com/{owner}/{repo}/commit/{sha}

    if !url.contains("github.com") {
        return Err("Not a GitHub URL".to_string());
    }

    // Strip trailing slashes
    let url = url.trim_end_matches('/');

    // Parse path segments after github.com
    let path = url
        .split("github.com/")
        .nth(1)
        .ok_or("Invalid GitHub URL")?;

    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() < 2 {
        return Err("Could not parse owner/repo from URL".to_string());
    }

    let owner = parts[0];
    let repo = parts[1];

    let md = match parts.get(2) {
        None => {
            // Just a repo link
            format!("[{owner}/{repo}]({url})")
        }
        Some(&"blob") if parts.len() >= 5 => {
            // File link: /blob/{branch}/{path...}
            let file_path = parts[4..].join("/");
            let branch = parts[3];
            format!("[{owner}/{repo}: {file_path} ({branch})]({url})")
        }
        Some(&"tree") if parts.len() >= 4 => {
            // Directory/branch link
            let branch = parts[3];
            let sub = if parts.len() > 4 {
                format!("/{}", parts[4..].join("/"))
            } else {
                String::new()
            };
            format!("[{owner}/{repo}{sub} ({branch})]({url})")
        }
        Some(&"issues") if parts.len() >= 4 => {
            let number = parts[3];
            format!("[{owner}/{repo}#{number}]({url})")
        }
        Some(&"pull") if parts.len() >= 4 => {
            let number = parts[3];
            format!("[{owner}/{repo} PR#{number}]({url})")
        }
        Some(&"commit") if parts.len() >= 4 => {
            let sha = &parts[3][..7.min(parts[3].len())];
            format!("[{owner}/{repo}@{sha}]({url})")
        }
        Some(&"releases") if parts.get(4) == Some(&"tag") && parts.len() >= 6 => {
            let tag = parts[5];
            format!("[{owner}/{repo} {tag}]({url})")
        }
        _ => {
            // Fallback: just wrap it
            format!("[{owner}/{repo}]({url})")
        }
    };

    Ok(md)
}

async fn convert(
    Json(payload): Json<ConvertRequest>,
) -> Result<ResponseJson<ConvertResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    match github_url_to_markdown(&payload.url) {
        Ok(markdown) => Ok(ResponseJson(ConvertResponse { markdown })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse { error: e }),
        )),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/convert", post(convert));

    let addr = "0.0.0.0:3000";
    println!("gh2md API listening on http://{addr}");
    println!();
    println!("Usage:");
    println!("  curl -s -X POST http://localhost:3000/convert \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"url\": \"https://github.com/owner/repo\"}}' | jq -r .markdown");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
