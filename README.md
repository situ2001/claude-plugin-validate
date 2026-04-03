# claude-plugin-validate

Deferring plugin validation until **RUNTIME** is painful, especially in **TEAM DEVELOPMENT**: teammates can hit errors during **INSTALLATION**, which increases **ADOPTION FRICTION**, and malformed plugin content may be silently ignored. So why not **SHIFT VALIDATION LEFT** and catch these issues before release?

---

This tool validates Claude Code plugin manifest schema (`plugin.json`) and
plugin content schemas (markdown frontmatter and `hooks/hooks.json` structure).

## Features

- Validate `plugin.json` schema + semantic constraints
- Optional validation for plugin markdown component frontmatter
- Optional hooks JSON structural validation
- CLI text and JSON output

## CLI

Installation

```bash
cargo install claude-plugin-validate
```

```bash
claude-plugin-validate /path/to/plugin.json
claude-plugin-validate /path/to/plugins-dir
claude-plugin-validate /path/to/plugins-dir --all
```

Exit codes:

- `0`: all valid
- `1`: one or more validation failures
- `2`: runtime/argument errors

## Run Example

Basic validation

```bash
❯ claude-plugin-validate ./plugins
OK  ./plugins/agent-sdk-dev/.claude-plugin/plugin.json
OK  ./plugins/claude-opus-4-5-migration/.claude-plugin/plugin.json
OK  ./plugins/code-review/.claude-plugin/plugin.json
OK  ./plugins/commit-commands/.claude-plugin/plugin.json
OK  ./plugins/explanatory-output-style/.claude-plugin/plugin.json
OK  ./plugins/feature-dev/.claude-plugin/plugin.json
OK  ./plugins/frontend-design/.claude-plugin/plugin.json
OK  ./plugins/hookify/.claude-plugin/plugin.json
OK  ./plugins/learning-output-style/.claude-plugin/plugin.json
OK  ./plugins/pr-review-toolkit/.claude-plugin/plugin.json
OK  ./plugins/ralph-wiggum/.claude-plugin/plugin.json
OK  ./plugins/security-guidance/.claude-plugin/plugin.json

Summary: 12/12 valid
```

Full validation

```bash
❯ claude-plugin-validate ./plugins --all
OK  ./plugins/agent-sdk-dev/.claude-plugin/plugin.json
OK  ./plugins/claude-opus-4-5-migration/.claude-plugin/plugin.json
OK  ./plugins/code-review/.claude-plugin/plugin.json
OK  ./plugins/commit-commands/.claude-plugin/plugin.json
OK  ./plugins/explanatory-output-style/.claude-plugin/plugin.json
OK  ./plugins/feature-dev/.claude-plugin/plugin.json
OK  ./plugins/frontend-design/.claude-plugin/plugin.json
OK  ./plugins/hookify/.claude-plugin/plugin.json
ERR ./plugins/hookify/agents/conversation-analyzer.md
  - frontmatter: YAML frontmatter failed to parse: mapping values are not allowed in this context at line 2 column 124. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
OK  ./plugins/learning-output-style/.claude-plugin/plugin.json
OK  ./plugins/pr-review-toolkit/.claude-plugin/plugin.json
ERR ./plugins/pr-review-toolkit/agents/code-reviewer.md
  - frontmatter: YAML frontmatter failed to parse: mapping values are not allowed in this context at line 2 column 717. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
ERR ./plugins/pr-review-toolkit/agents/code-simplifier.md
  - frontmatter: YAML frontmatter failed to parse: could not find expected ':' at line 7 column 1, while scanning a simple key at line 6 column 1. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
ERR ./plugins/pr-review-toolkit/agents/comment-analyzer.md
  - frontmatter: YAML frontmatter failed to parse: mapping values are not allowed in this context at line 2 column 140. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
ERR ./plugins/pr-review-toolkit/agents/pr-test-analyzer.md
  - frontmatter: YAML frontmatter failed to parse: mapping values are not allowed in this context at line 2 column 272. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
ERR ./plugins/pr-review-toolkit/agents/silent-failure-hunter.md
  - frontmatter: YAML frontmatter failed to parse: mapping values are not allowed in this context at line 2 column 393. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
ERR ./plugins/pr-review-toolkit/agents/type-design-analyzer.md
  - frontmatter: YAML frontmatter failed to parse: mapping values are not allowed in this context at line 2 column 111. At runtime this agent loads with empty metadata (all frontmatter fields silently dropped).
OK  ./plugins/ralph-wiggum/.claude-plugin/plugin.json
OK  ./plugins/security-guidance/.claude-plugin/plugin.json

Summary: 12/19 valid
```
