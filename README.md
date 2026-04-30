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

```bash
npx angular-docs-to-md https://angular.dev/guide/signals/linked-signal
npx angular-docs-to-md https://angular.dev/guide/components --examples 2
npx angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
```

## Claude Code skill

Install the skill so Claude can fetch Angular docs on demand:

```bash
npx skills add pyyding/angular-docs-to-md
```

Then use it from any Claude Code session:

```
/angular-docs-to-md https://angular.dev/guide/signals
/angular-docs-to-md https://angular.dev/guide/components --examples 2
/angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
```
