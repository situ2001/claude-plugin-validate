# claude-plugin-validate

This tool validates Claude Code plugin manifest schema (`plugin.json`) and
plugin content schemas (markdown frontmatter and `hooks/hooks.json` structure).

## Inputs

- Logic spec: `spec/VALIDATOR_SPEC.md`
- Fixture set for behavior checks: `fixtures/`

## Features

- Validate `plugin.json` schema + semantic constraints
- Optional validation for plugin markdown component frontmatter
- Optional hooks JSON structural validation
- CLI text and JSON output

## CLI

```bash
cargo run -- /path/to/plugin.json
cargo run -- /path/to/plugins-dir
cargo run -- /path/to/plugins-dir --json
cargo run -- /path/to/plugins-dir --all
```

Exit codes:

- `0`: all valid
- `1`: one or more validation failures
- `2`: runtime/argument errors
