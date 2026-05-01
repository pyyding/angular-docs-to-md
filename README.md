# angular-docs-to-md

A CLI tool that fetches an [angular.dev](https://angular.dev) documentation page and converts it to clean Markdown, with custom HTML elements expanded into readable content.

## Usage

```bash
npx angular-docs-to-md <URL> [options]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--examples N` | `1` | Tab examples to expand per `<docs-tab-group>` |
| `--no-header` | — | Skip `<docs-decorative-header>` parsing |
| `--no-pills` | — | Skip `<docs-pill-row>` parsing |
| `--refresh` | — | Re-fetch even if a cached file exists |
| `--no-cache` | — | Skip the cache entirely (no read, no write) |

```bash
npx angular-docs-to-md https://angular.dev/guide/signals/linked-signal
npx angular-docs-to-md https://angular.dev/guide/components --examples 2
npx angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
```

## Caching

Results are saved to `.cache/` next to the CLI and reused for 24 hours. Run with `--refresh` to force a re-fetch, or `--no-cache` to skip the cache for that run.

## Agent skill

Install via the [skills CLI](https://skills.sh):

```bash
npx skills add pyyding/angular-docs-to-md
```

Then invoke from any agent session:

```
/angular-docs-to-md https://angular.dev/guide/signals
/angular-docs-to-md https://angular.dev/guide/aria/menu --examples 2
/angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
```
