# angular-docs-to-md

A lightweight REST API that converts GitHub URLs and Angular documentation pages to Markdown. Built with Rust + Axum.

## Endpoints

### `GET /gh-to-md`

Converts a GitHub URL to a Markdown snippet.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `url` | yes | Any GitHub URL (repo, issue, PR, commit, file, tree, release) |
| `scope` | no | Set to `current-dir` to fetch all files in the same directory as a blob URL |

**Examples**

```bash
# Repo
curl "http://localhost:3000/gh-to-md?url=https://github.com/angular/angular"

# Issue
curl "http://localhost:3000/gh-to-md?url=https://github.com/angular/angular/issues/123"

# PR
curl "http://localhost:3000/gh-to-md?url=https://github.com/angular/angular/pull/456"

# File
curl "http://localhost:3000/gh-to-md?url=https://github.com/angular/angular/blob/main/README.md"

# All files in the same directory
curl "http://localhost:3000/gh-to-md?url=https://github.com/angular/angular/blob/main/src/app/app.component.ts&scope=current-dir"
```

---

### `GET /angular-docs-to-md`

Fetches an [angular.dev](https://angular.dev) documentation page and returns it as clean Markdown, with custom HTML elements expanded into readable content.

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `url` | yes | — | An `https://angular.dev/` URL |
| `examples_per_group` | no | `1` | How many tabs to expand per `<docs-tab-group>` |
| `parse_header_html` | no | `true` | Replace `<docs-decorative-header>` with a Markdown `h1` |
| `parse_pills` | no | `true` | Replace `<docs-pill-row>` with Markdown links |

**Examples**

```bash
curl "http://localhost:3000/angular-docs-to-md?url=https://angular.dev/guide/signals"

curl "http://localhost:3000/angular-docs-to-md?url=https://angular.dev/guide/routing&examples_per_group=2"
```

---

## Running

```bash
cargo run
```

Optionally set a GitHub token to raise the API rate limit from 60 to 5000 requests/hour:

```bash
GITHUB_TOKEN=ghp_... cargo run
```

## Building

```bash
cargo build --release
./target/release/angular-docs-to-md
```
