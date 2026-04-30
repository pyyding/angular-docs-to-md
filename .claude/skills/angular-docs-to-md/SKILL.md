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

### 1. Build the binary if needed

```bash
ls /Users/kaspar/gh-2-md/target/release/angular-docs-to-md 2>/dev/null && echo "EXISTS" || echo "MISSING"
```

If `MISSING`:
```bash
cargo build --release --manifest-path /Users/kaspar/gh-2-md/Cargo.toml
```

### 2. Run the binary

```bash
/Users/kaspar/gh-2-md/target/release/angular-docs-to-md <URL> [flags]
```

Flags:
- `--examples N` — tab examples to expand per group (default: 1)
- `--no-header` — skip `<docs-decorative-header>` parsing
- `--no-pills` — skip `<docs-pill-row>` parsing

### 3. Return the result

Output the Markdown directly. If the binary exits with an error, show the message and suggest fixes.
