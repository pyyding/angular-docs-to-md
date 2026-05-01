---
name: angular-docs-to-md
description: Fetch an angular.dev page and convert it to clean Markdown
---

Fetch an Angular documentation page from `angular.dev` and convert it to clean Markdown, expanding code examples inline. Results are cached locally for 24 hours so repeat lookups are instant.

## How to invoke

```
/angular-docs-to-md <URL> [flags]
```

Examples:
```
/angular-docs-to-md https://angular.dev/guide/components
/angular-docs-to-md https://angular.dev/guide/signals --examples 3
/angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
/angular-docs-to-md https://angular.dev/guide/components --refresh
```

## Steps

### 1. Run

```bash
npx angular-docs-to-md <URL> [flags]
```

Flags:
- `--examples N` — tab examples to expand per group (default: 1)
- `--no-header` — skip `<docs-decorative-header>` parsing
- `--no-pills` — skip `<docs-pill-row>` parsing
- `--refresh` — re-fetch even if a cached file exists
- `--no-cache` — skip the cache entirely for this run

### 2. Return the result

Output the Markdown directly. If the command exits with an error, show the message and suggest fixes.
