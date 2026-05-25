---
name: warpctrl
description: Use the Warp Control CLI to inspect or control a running local Warp app during dogfood. Use when operating Warp product surfaces through warpctrl, checking local-control availability, targeting a running Warp instance, or evaluating planned warpctrl commands against the implemented catalog.
user-invocable: true
---

# Warp Control CLI

Use `warpctrl` to operate an already-running local Warp app through the approved local-control command surface. Prefer native tools for code editing, file content reads or writes, shell execution, web requests, and MCP calls. Use `warpctrl` when the task is about Warp product surfaces: windows, tabs, panes, visible app state, Warp Drive views, settings surfaces, or future permissioned Drive actions.

## Current implemented commands

The current dogfood foundation slice implements only:

- `warpctrl instance list`
- `warpctrl app ping [--instance <id>|--pid <pid>]`
- `warpctrl app version [--instance <id>|--pid <pid>]`
- `warpctrl tab create [--instance <id>|--pid <pid>]`
- `warpctrl completions [shell]`

Treat all other commands as planned unless `warpctrl help`, `warpctrl instance list --output-format json`, or the selected app's action catalog advertises them as implemented.

## Targeting workflow

1. List running compatible instances:

   ```bash
   warpctrl instance list
   ```

2. If exactly one compatible instance is available, run the command directly:

   ```bash
   warpctrl tab create
   ```

3. If multiple instances are available, target one explicitly:

   ```bash
   warpctrl tab create --instance <instance_id>
   ```

4. Use JSON output for automation:

   ```bash
   warpctrl --output-format json app version --instance <instance_id>
   ```

## Permission posture

- Metadata reads such as `instance.list`, `app.ping`, and `app.version` require read-metadata authority.
- `tab.create` is an app-state mutation. It changes visible Warp UI state but does not directly mutate terminal contents, local files, or Warp Drive data.
- Underlying-data reads and mutations require stronger permissions and, for Warp Drive or execution-backed actions, authenticated-user authority. Do not attempt to bypass the CLI by calling local-control HTTP endpoints directly.

## File and project boundaries

`warpctrl` does not provide local file content commands. Do not use or invent commands such as `file read`, `file write`, `file append`, or `file delete`.

The approved file/project scope is app-state only:

- `file open <path>` is planned to open a file in Warp's visible editor surface.
- `file list` is planned to list files currently open in Warp editor state.
- `project open`, `project list`, and `project active` are planned to operate Warp project/workspace state.

Use the agent's native file tools or ordinary shell commands for local filesystem content reads, writes, appends, and deletes.

## Warp Drive sharing boundaries

Warp Drive sharing v0 has two paths:

- `drive object share open <id>` is a planned app-state mutation that opens the share dialog for user review without changing sharing state.
- `drive object share-to-team <id>` is the only planned direct native sharing mutation. It makes a personal object available to the current user's team and requires authenticated-user plus underlying-data-mutation authority.

Do not use `warpctrl` for arbitrary ACL editing, external sharing, named-user sharing, public links, accepted-command submission, agent-prompt submission, or arbitrary internal dispatch.

## Handling unsupported commands

When a command is in the product spec but not implemented in the selected app build, report it as planned or unsupported rather than trying nearby internal actions. A parser error, `unsupported_action`, `not_allowlisted`, `insufficient_permissions`, `authenticated_user_required`, or `authenticated_user_unavailable` response is an expected boundary, not a reason to bypass the catalog.
