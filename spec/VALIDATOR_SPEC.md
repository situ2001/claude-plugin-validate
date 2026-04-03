# Plugin Validator Spec (Behavior Contract)

This document defines behavior-only constraints for validating plugin manifests
and plugin markdown component files. It is intentionally implementation-neutral.

## 1. Manifest Root

- Input must be a JSON object.
- Required field: `name`.

## 2. Name

- `name` must be a non-empty string.
- Allowed charset: `^[a-z0-9][-a-z0-9._]*$` (case-insensitive).

## 3. Metadata

- `version`, `description`, `repository`, `license`: optional string.
- `homepage`: optional URL string.
- `author`: optional object with required non-empty `name`, optional `email`/`url` strings.
- `keywords`: optional string array.

## 4. Dependencies

`dependencies` is optional array of either:

- string format: `name`, `name@marketplace`, `name@marketplace@^version`
- object format: `{ name, marketplace? }`

Validation:

- name-like identifiers use `^[a-z0-9][-a-z0-9._]*$`.

## 5. Path Rules

- relative path must start with `./`.
- JSON path must be relative path and end with `.json`.
- markdown path must be relative path and end with `.md`.

## 6. Hooks Field

`hooks` supports:

- JSON path string
- hooks object `{ event -> matcher[] }`
- array containing path/object variants

Hook event enum:

- PreToolUse, PostToolUse, PostToolUseFailure, Notification, UserPromptSubmit,
  SessionStart, SessionEnd, Stop, StopFailure, SubagentStart, SubagentStop,
  PreCompact, PostCompact, PermissionRequest, PermissionDenied, Setup,
  TeammateIdle, TaskCreated, TaskCompleted, Elicitation, ElicitationResult,
  ConfigChange, WorktreeCreate, WorktreeRemove, InstructionsLoaded,
  CwdChanged, FileChanged

Hook matcher:

- optional `matcher: string`
- required `hooks: HookCommand[]`

HookCommand type is discriminated by `type`:

- `command`: requires `command: string`; optional `shell` in {bash,powershell}
- `prompt`: requires `prompt: string`
- `agent`: requires `prompt: string`
- `http`: requires `url: valid URL`

Shared optional fields:

- `if: string`
- `timeout: number > 0`
- `statusMessage: string`
- `once: boolean`
- `async: boolean` (command only)
- `asyncRewake: boolean` (command only)
- `model: string` (prompt/agent)
- `headers: string map` (http)
- `allowedEnvVars: string[]` (http)

## 7. Commands Field

`commands` supports:

- path string
- array of path strings
- object map `commandName -> metadata`

Command metadata:

- exactly one of `source` or `content` must exist
- `source` must be relative path
- `content` must be string
- optional string fields: `description`, `argumentHint`, `model`
- optional `allowedTools: string[]`

## 8. Agents / Skills / OutputStyles

- `agents`: path or path array, markdown-only
- `skills`: path or path array
- `outputStyles`: path or path array

## 9. MCP Servers Field

`mcpServers` supports:

- JSON path
- `.mcpb` / `.dxt` relative path or URL
- map `name -> server config`
- array of mixed variants above

Supported config `type`:

- stdio (default when absent)
- sse
- sse-ide
- ws-ide
- http
- ws
- sdk
- claudeai-proxy

Each type has required field checks and optional field type checks.
OAuth object checks:

- `clientId: string?`
- `callbackPort: positive integer?`
- `authServerMetadataUrl: https URL?`
- `xaa: boolean?`

## 10. LSP Servers Field

`lspServers` supports:

- JSON path
- map `name -> config`
- array mixing path/map

LSP config checks:

- required `command: non-empty string`
- command cannot include spaces unless absolute path style (`/` prefix)
- required non-empty map `extensionToLanguage`
- extension key must start with `.`
- language value must be non-empty string
- optional `transport` in {stdio, socket}
- optional maps/ints/bools with strict type checks for known fields

## 11. userConfig

`userConfig` is optional object map `key -> option`.

Key regex:

- `^[A-Za-z_]\w*$`

Option strict keys:

- required: `type`, `title`, `description`
- optional: `required`, `default`, `multiple`, `sensitive`, `min`, `max`
- reject unknown keys

`type` enum:

- string, number, boolean, directory, file

`default` allowed:

- string | number | boolean | string[]

## 12. channels

`channels` optional array of strict objects:

- required non-empty `server: string`
- optional `displayName: string`
- optional `userConfig: object` using same option schema as top-level userConfig
- reject unknown keys

## 13. settings

- optional object map

## 14. Markdown Component Validation

For command/agent/skill markdown files:

- if no frontmatter block: warning
- if frontmatter parse fails: error
- if frontmatter is non-object: error
- description missing: warning
- description non-scalar: error
- name present but non-string: error
- allowed-tools: string or string[] only
- shell present: string in {bash,powershell} only

