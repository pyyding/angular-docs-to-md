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

### 1. Run

```bash
npx angular-docs-to-md <URL> [flags]
```

Flags:
- `--examples N` — tab examples to expand per group (default: 1)
- `--no-header` — skip `<docs-decorative-header>` parsing
- `--no-pills` — skip `<docs-pill-row>` parsing

### 2. Return the result

Output the Markdown directly. If the command exits with an error, show the message and suggest fixes.
