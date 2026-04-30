# angular-docs-to-md

A CLI tool that fetches an [angular.dev](https://angular.dev) documentation page and converts it to clean Markdown, with custom HTML elements expanded into readable content. Built with Rust.

## Usage

```bash
angular-docs-to-md <URL> [options]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--examples N` | `1` | Tab examples to expand per `<docs-tab-group>` |
| `--no-header` | — | Skip `<docs-decorative-header>` parsing |
| `--no-pills` | — | Skip `<docs-pill-row>` parsing |

**Examples**

```bash
angular-docs-to-md https://angular.dev/guide/signals/linked-signal

angular-docs-to-md https://angular.dev/guide/components --examples 2

angular-docs-to-md https://angular.dev/guide/templates/pipes --no-pills
```

## Build

```bash
cargo build --release
./target/release/angular-docs-to-md https://angular.dev/guide/signals
```

## Claude Code skill

Invoke directly from Claude Code with `/angular-docs-to-md <url>` — the skill is defined in `.claude/skills/angular-docs-to-md.md`.
