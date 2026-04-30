use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;

// ── compiled regexes (once per process) ──────────────────────────────────────

static HEADER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<docs-decorative-header title="([^"]+)"[^>]*>.*?</docs-decorative-header>"#)
        .unwrap()
});

static PILL_ROW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<docs-pill-row>(.*?)</docs-pill-row>").unwrap()
});

static PILL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<docs-pill href="([^"]+)" title="([^"]+)"/>"#).unwrap()
});

static TAB_GROUP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<docs-tab-group>(.+?)</docs-tab-group>").unwrap()
});

static TAB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<docs-tab label="([^"]+)">(.+?)</docs-tab>"#).unwrap()
});

static CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<docs-code header="([^"]+)" path="([^"]+)"/>"#).unwrap()
});

// ── helpers ───────────────────────────────────────────────────────────────────

async fn fetch_text(client: &Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .header("User-Agent", "gh2md/0.1")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

fn replace_decorative_headers(markdown: &str) -> String {
    HEADER_RE
        .replace_all(markdown, |caps: &regex::Captures| format!("# {}", &caps[1]))
        .into_owned()
}

fn replace_pill_rows(markdown: &str) -> String {
    PILL_ROW_RE
        .replace_all(markdown, |row_cap: &regex::Captures| {
            let inner = &row_cap[1];
            let links: String = PILL_RE
                .captures_iter(inner)
                .map(|c| format!("- [{}]({})\n", &c[2], &c[1]))
                .collect();
            links.trim_end().to_string()
        })
        .into_owned()
}

async fn expand_tab_groups(
    markdown: &str,
    examples_per_group: usize,
    client: &Client,
) -> Result<String, String> {
    // Collect (start, end, replacement) tuples first so we can apply in reverse.
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    for group_cap in TAB_GROUP_RE.captures_iter(markdown) {
        let full     = group_cap.get(0).unwrap();
        let inner    = group_cap.get(1).unwrap().as_str();
        let mut group_md = String::new();

        for tab_cap in TAB_RE.captures_iter(inner).take(examples_per_group) {
            let label       = &tab_cap[1];
            let tab_content = &tab_cap[2];

            group_md.push_str(&format!("**Example: {label}**\n\n"));

            for code_cap in CODE_RE.captures_iter(tab_content) {
                let header  = &code_cap[1];
                let path    = &code_cap[2];
                let raw_url = format!(
                    "https://raw.githubusercontent.com/angular/angular/main/{path}"
                );
                let content = fetch_text(client, &raw_url).await?;
                let ext     = header.rsplit('.').next().unwrap_or("");
                group_md.push_str(&format!("```{ext}\n// {header}\n{content}\n```\n\n"));
            }
        }

        replacements.push((full.start(), full.end(), group_md.trim_end().to_string()));
    }

    // Apply in reverse so byte offsets stay valid.
    let mut result = markdown.to_string();
    for (start, end, rep) in replacements.into_iter().rev() {
        result.replace_range(start..end, &rep);
    }

    Ok(result)
}

// ── public API ────────────────────────────────────────────────────────────────

pub async fn convert_angular_docs(
    client: &Client,
    url: &str,
    examples_per_group: usize,
    parse_header_html: bool,
    parse_pills: bool,
) -> Result<String, String> {
    let url = url.trim().trim_end_matches('/');

    let path = url
        .strip_prefix("https://angular.dev/")
        .ok_or("Not an angular.dev URL")?;

    let raw_url = format!(
        "https://raw.githubusercontent.com/angular/angular/main/adev/src/content/{path}.md"
    );

    let body = fetch_text(client, &raw_url).await?;
    let body = if parse_header_html { replace_decorative_headers(&body) } else { body };
    let body = if parse_pills        { replace_pill_rows(&body)          } else { body };

    expand_tab_groups(&body, examples_per_group, client).await
}
