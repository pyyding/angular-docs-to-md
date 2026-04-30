---
name: angular-docs-to-md
description: Fetch an angular.dev page and convert it to clean Markdown
---

Fetch an Angular documentation page from `angular.dev` and convert it to clean Markdown, expanding code examples inline.

## How to invoke

```
/angular-docs-to-md <URL> [flags]
```

Examples:
```
/angular-docs-to-md https://angular.dev/guide/components
/angular-docs-to-md https://angular.dev/guide/signals --examples 3
/angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
```

## Steps

### 1. Locate the binary

```bash
which angular-docs-to-md 2>/dev/null && echo "IN_PATH" || echo "NOT_IN_PATH"
```

If `IN_PATH`, skip to step 2.

If `NOT_IN_PATH`, check for the repo via the env var `GH_2_MD_PATH`:

```bash
ls "${GH_2_MD_PATH}/target/release/angular-docs-to-md" 2>/dev/null && echo "EXISTS" || echo "MISSING"
```

If `MISSING` (and `GH_2_MD_PATH` is set), build it:
```bash
cargo build --release --manifest-path "${GH_2_MD_PATH}/Cargo.toml"
```

If `GH_2_MD_PATH` is not set and the binary is not in PATH, tell the user to either:
- Install the binary and add it to `PATH`, or
- Set `GH_2_MD_PATH` to the root of the `gh-2-md` repo in their shell profile or `.claude/settings.json` env block.

### 2. Run the binary

```bash
# Uses PATH if available, otherwise falls back to the local build
BINARY=$(which angular-docs-to-md 2>/dev/null || echo "${GH_2_MD_PATH}/target/release/angular-docs-to-md")
"$BINARY" <URL> [flags]
```

Flags:
- `--examples N` — tab examples to expand per group (default: 1)
- `--no-header` — skip `<docs-decorative-header>` parsing
- `--no-pills` — skip `<docs-pill-row>` parsing

### 3. Return the result

Output the Markdown directly. If the binary exits with an error, show the message and suggest fixes.
