# Summary
Warp ships a local control CLI, provisionally named `warpctrl`, that lets agents, developers, and scripts operate running Warp app processes through a typed, allowlisted command surface. `warpctrl` is an Oz-style wrapper script that invokes the existing channel-specific Warp binary in control mode rather than a separate standalone binary.
The public catalog contains exactly **75 actions** organized around stable user-facing nouns. **72 actions** are default-authorized once the user enables Scripting. **3 destructive close actions** (`window.close`, `tab.close`, `pane.close`) require one-shot in-app confirmation before executing. `block.list` is intentionally absent from the catalog. Input-staging commands place text in the input buffer but never submit it.
All callers are external same-user processes. There is no inside-Warp/outside-Warp distinction, no verified-terminal invocation context, and no authenticated-user identity layer. Security relies on owner-only filesystem discovery, same-user Unix credential broker with kernel peer credentials, short-lived instance-bound exact-action credentials, loopback HTTP transport, and app-side enforcement.
## Problem
Warp has rich interactive actions reachable through UI, keybindings, menus, and deeplinks. Agents can use native tools for files, code, shell commands, and MCP calls, but they cannot reliably operate Warp's own product surfaces: arranging workspaces, focusing panes, opening Warp Drive views, presenting settings, or recovering from ambiguous UI state. Developers cannot compose those actions into shell scripts, demos, or automation workflows, and there is no general local protocol for addressing a specific running Warp instance, window, tab, pane, or session.
## Goals
- Provide a first-class, scriptable `warpctrl` command for controlling running Warp app processes.
- Make Warp's UI and app state available to agents through a typed, permissioned control plane instead of brittle screen automation.
- Keep CLI startup lightweight by avoiding GUI-app startup for routine control commands.
- Keep the surface allowlisted and finite: exactly 75 named actions, no arbitrary internal dispatch.
- Make targeting explicit and deterministic across multiple Warp processes, windows, tabs, panes, and sessions.
- Use a simple enabled/disabled Scripting setting rather than multi-mode invocation-context policies.
## Non-goals
- Replacing the Oz CLI or mixing cloud-agent management into this CLI.
- Exposing every internal app action, debug action, or privileged state mutation.
- Treating the CLI as a general RPC escape hatch into Warp internals.
- Replacing native agent tools for code editing, file operations, shell execution, or MCP calls.
- Providing an authenticated-user identity layer, verified-terminal invocation proof, or invocation-context distinction.
- Terminal command execution, accepted-command submission, or agent-prompt submission.
- Warp Drive data mutations, cloud-backed state mutations, or sharing operations.
- Local file content reads, writes, or filesystem-content mutations.
## Primary user stories
1. **Agent workspace orchestration.** An agent inspects current Warp state, creates or reuses an appropriate window/tab layout, splits panes, names and focuses targets, and leaves the workspace in a readable task-shaped state. The agent continues to use native tools for code edits, file I/O, shell execution, and MCP calls.
2. **Existing-session debugging and repair.** An agent understands Warp-specific UI and session structure before acting: which instance/window/tab/pane/session is active, whether the relevant pane still exists, which surface is focused, and which selector to use for follow-up actions.
3. **Deterministic demos and walkthroughs.** A script puts Warp into a known presentation state: theme, zoom, windows, tabs, panes, focused targets, panels, and surfaces. The walkthrough advances using structured target IDs and recovers from stale or missing targets.
4. **Personalization and preference migration.** An agent inspects settings, proposes Warp equivalents from other tools, applies allowlisted changes, and reports unsupported mappings explicitly.
## Behavior
1. The CLI operates only on running local Warp app processes. If no compatible process is available, it exits non-zero with a structured error.
2. The CLI exposes only the 75 explicitly allowlisted actions. Unknown, unsupported, or non-allowlisted requests fail with structured errors and are never forwarded to arbitrary internal dispatch.
3. Every successful mutating request identifies the Warp process instance, resolved target, and a success payload suitable for JSON output.
4. Every failure identifies a stable machine-readable error code, a human-readable explanation, and any selector that was ambiguous, missing, stale, or invalid.
5. The CLI supports human-readable output by default and JSON output for scripts with stable field names.
6. Process discovery and instance selection:
   - `warpctrl instance list` returns all reachable local Warp app processes.
   - Each process has an opaque `instance_id`, channel/build identity, and display metadata.
   - If exactly one compatible process is available, commands target it implicitly.
   - If multiple compatible processes are available and no single clearly active instance exists, the CLI fails and asks for an explicit `--instance` selector.
7. Target introspection:
   - `warpctrl window list`, `warpctrl tab list`, `warpctrl pane list`, `warpctrl session list`, `warpctrl app active`.
   - These return opaque protocol-facing IDs and metadata for subsequent commands.
8. The target selector model is hierarchical: instance → window → tab → pane → session. Non-hierarchical selectors (files, surfaces) resolve inside the selected instance.
9. Every selector family supports an ergonomic `active` form. For window-scoped mutations, an omitted window selector may fall back to the sole existing window. Zero windows returns `missing_target`; multiple windows without an active one returns `ambiguous_target`.
10. Every selector family supports explicit opaque IDs and may support scoped indices or titles for interactive use. IDs remain the preferred automation surface.
11. When a command omits lower-level selectors, it resolves them from the higher-level context using active defaults.
12. When an explicitly supplied target disappears between discovery and execution, the request fails with `stale_target`. The CLI never silently chooses a different target.
13. The protocol is command-oriented: each action has a named command, validated parameters, and defined target scope.
## Scripting setting
Warp adds a new top-level Settings pane page named **Scripting**. The page contains a single toggle for local control:
- **Enabled** (default): same-user processes may request exact-action credentials from the broker and send control requests to the loopback listener.
- **Disabled**: no same-user process can receive local-control credentials. The control listener does not accept requests. Discovery records contain no actionable endpoint.
The authoritative value is stored in protected local storage (macOS Keychain, or owner-only secure storage on Linux). It is never synced, never appears in `settings.toml` or generated schemas, and cannot be changed by `warpctrl`, config files, or direct protocol requests. Only the Warp app through Settings > Scripting can change it. The default is enabled. Disabling Scripting immediately prevents new credential issuance and invalidates outstanding credentials.
## One-shot close confirmation
Three destructive actions require one-shot in-app confirmation before executing:
- `window.close`
- `tab.close`
- `pane.close`
When the app bridge receives one of these actions, it presents a brief in-app confirmation to the user. The user must approve the close before it executes. If the user dismisses the confirmation, the action fails with `user_confirmation_denied`. If the confirmation times out without a response, the action fails with `user_confirmation_expired`. The confirmation is per-invocation; there is no persistent "always allow" option for close actions.
All other 72 actions execute immediately once the credential is validated.
## Input staging
The two input commands (`input.insert`, `input.replace`) only stage or edit text in the terminal input buffer. They never submit the buffer, press Enter, or execute a command. There is no `input.run`, `input.get`, `input.clear`, or `input.mode.set` action in the catalog. Terminal command execution is not part of this product surface.
## Action catalog
The public catalog contains exactly 75 actions. The Block, Auth, Drive, and History families are entirely absent. Input is limited to `input.insert` and `input.replace`. Actions are organized by noun and use the exact dotted names from the authoritative `ActionKind` catalog.
### Instance (2 actions)
All default-authorized.
- `instance.list` — list reachable Warp app processes.
- `instance.inspect` — metadata for one instance.
### App (4 actions)
All default-authorized.
- `app.ping` — health check for the selected instance.
- `app.version` — build/channel/version metadata.
- `app.active` — the active instance/window/tab/pane/session chain.
- `app.focus` — bring the selected Warp app to the foreground.
### Capability (2 actions)
All default-authorized.
- `capability.list` — list capabilities supported by the selected instance.
- `capability.inspect` — metadata for one capability.
### Window (5 actions)
4 default-authorized, 1 one-shot confirmation.
- `window.list` — list windows in the selected instance.
- `window.inspect` — metadata for one window.
- `window.create` — create a new window.
- `window.focus` — focus a target window.
- `window.close` — close a target window. **Requires one-shot confirmation.**
### Tab (10 actions)
9 default-authorized, 1 one-shot confirmation.
- `tab.list` — list tabs in the selected window.
- `tab.inspect` — metadata for one tab.
- `tab.create` — create a new terminal tab.
- `tab.activate` — activate a target tab.
- `tab.move` — move a tab left or right.
- `tab.close` — close a target tab. **Requires one-shot confirmation.**
- `tab.rename` — rename a tab.
- `tab.reset_name` — reset a tab title to the default.
- `tab.color.set` — set the active-tab color.
- `tab.color.clear` — clear the active-tab color.
### Pane (11 actions)
10 default-authorized, 1 one-shot confirmation.
- `pane.list` — list panes in the selected tab.
- `pane.inspect` — metadata for one pane.
- `pane.split` — split a pane in a direction (left, right, up, down).
- `pane.focus` — focus a target pane.
- `pane.navigate` — navigate focus between panes (left, right, up, down).
- `pane.resize` — resize pane dividers in a direction.
- `pane.maximize` — toggle maximize for a pane.
- `pane.unmaximize` — restore a maximized pane.
- `pane.close` — close a target pane. **Requires one-shot confirmation.**
- `pane.rename` — rename a pane.
- `pane.reset_name` — reset a pane title to the default.
### Session (6 actions)
All default-authorized.
- `session.list` — list sessions in the selected pane.
- `session.inspect` — metadata for one session.
- `session.activate` — activate a target session.
- `session.previous` — cycle to the previous session.
- `session.next` — cycle to the next session.
- `session.reopen_closed` — reopen the last closed session.
### Input (2 actions)
All default-authorized. **Input commands stage text only and never submit.**
- `input.insert` — insert text into the input buffer without executing.
- `input.replace` — replace the input buffer contents without executing.
### Theme (6 actions)
All default-authorized.
- `theme.list` — list available themes.
- `theme.get` — get the current theme.
- `theme.set` — set the current fixed theme.
- `theme.system.set` — toggle or set "follow system theme."
- `theme.light.set` — set the light-mode theme.
- `theme.dark.set` — set the dark-mode theme.
### Appearance (7 actions)
All default-authorized.
- `appearance.get` — get current appearance state (font size, zoom).
- `appearance.font_size.increase` — increase font size.
- `appearance.font_size.decrease` — decrease font size.
- `appearance.font_size.reset` — reset font size to default.
- `appearance.zoom.increase` — increase UI zoom.
- `appearance.zoom.decrease` — decrease UI zoom.
- `appearance.zoom.reset` — reset UI zoom to default.
### Setting (4 actions)
All default-authorized.
- `setting.list` — list allowlisted user-facing settings.
- `setting.get` — read an allowlisted setting value.
- `setting.set` — set an allowlisted setting to a validated value.
- `setting.toggle` — toggle an allowlisted boolean setting.
Private, debug-only, derived, and non-allowlisted settings are rejected with structured errors.
### Keybinding (2 actions)
All default-authorized.
- `keybinding.list` — list keybindings.
- `keybinding.get` — get a specific keybinding.
### Action (2 actions)
All default-authorized.
- `action.list` — list all 75 catalog actions with implementation status.
- `action.inspect` — metadata for one action.
### Surface (11 actions)
All default-authorized.
- `surface.settings.open` — open the settings surface, optionally to a specific page or search query.
- `surface.command_palette.open` — open or toggle the command palette with an optional initial query.
- `surface.command_search.open` — open or toggle command search.
- `surface.warp_drive.open` — open the Warp Drive panel.
- `surface.warp_drive.toggle` — toggle the Warp Drive panel.
- `surface.resource_center.toggle` — toggle the resource center.
- `surface.ai_assistant.toggle` — toggle the AI assistant panel.
- `surface.code_review.toggle` — toggle the code review panel.
- `surface.left_panel.toggle` — toggle the left panel.
- `surface.right_panel.toggle` — toggle the right panel.
- `surface.vertical_tabs.toggle` — toggle vertical tabs.
### File (1 action)
Default-authorized.
- `file.open` — open a file path in a Warp editor tab, optionally at a specific line and column. This is an app-state intent, not a filesystem-content operation.
### Excluded from the catalog
The following families and actions are entirely absent even when internal implementations exist:
- The entire Block family (`block.list`, `block.inspect`, `block.output`).
- The entire Auth family (`auth.status`, `auth.login`).
- The entire Drive family (all `drive.*` actions).
- The entire History family (`history.list`).
- `input.get`, `input.clear`, `input.mode.set`, `input.run`, and any form of terminal command execution or submission.
- `file.list` and any local file content reads, writes, or filesystem-content mutations.
- Accepted-command submission and agent-prompt submission.
- Crash, panic, heap-dump, token-copying, debug-reset, and developer/debug helpers.
- Arbitrary internal view dispatch by string.
- Arbitrary settings outside the allowlist.
## CLI command surface
Command names are noun-oriented and discoverable. Examples:
- `warpctrl instance list`
- `warpctrl app ping`
- `warpctrl app active`
- `warpctrl tab create`
- `warpctrl tab rename --tab <id> "Build logs"`
- `warpctrl window close --window <id>`
- `warpctrl pane split --direction right`
- `warpctrl input replace "cargo check"`
- `warpctrl theme set "Warp Dark"`
- `warpctrl setting set appearance.themes.system_theme true`
- `warpctrl file open src/main.rs --line 42`
### Targeting flags
- `--instance <instance_id>` and `--pid <pid>` select a running Warp process (mutually exclusive).
- `--window <active|opaque-id>`, `--window-index <n>`, and `--window-title <title>` select a window.
- `--tab <active|opaque-id>`, `--tab-index <n>`, and `--tab-title <title>` select a tab.
- `--pane <active|opaque-id>` and `--pane-index <n>` select a pane.
- `--session <active|opaque-id>` selects a session.
- `--output-format <pretty|json|text>` controls output shape.
Within a selector family, specifying more than one form is invalid. Handlers reject selector forms that they cannot resolve safely.
### Wire protocol
A request contains an action name from the catalog, a structured target selector, and validated parameters. A response contains success/failure status, resolved instance and target metadata, and result data or structured error data. The protocol is versioned.
## Error model
Every protocol or runtime failure identifies a stable machine-readable error code:
- `local_control_disabled` — Scripting is disabled.
- `unauthorized_local_client` — missing, malformed, expired, or invalid credential.
- `insufficient_permissions` — credential grants a different action.
- `user_confirmation_required` — action requires one-shot confirmation that has not been presented yet.
- `user_confirmation_denied` — user declined one-shot close confirmation.
- `user_confirmation_expired` — one-shot confirmation timed out without a response.
- `ambiguous_instance` — multiple instances, no unambiguous selection.
- `ambiguous_target` — multiple matching targets.
- `stale_target` — explicit target ID no longer exists.
- `missing_target` — no active or default target exists.
- `invalid_selector` — malformed selector syntax.
- `invalid_request` — malformed request body.
- `invalid_params` — invalid action-specific parameters.
- `unsupported_action` — action not implemented by this build.
- `not_allowlisted` — action intentionally excluded from public surface.
- `target_state_conflict` — target cannot support the requested action.
- `no_instance` — no reachable Warp instance found.
## Unsupported platforms
On platforms where the owner-only filesystem discovery, Unix credential broker, or equivalent authenticated broker transport are not available, `warpctrl` fails closed. It does not fall back to unauthenticated control or weaker credential models. Windows local-control publication remains disabled until discovery-record ACL enforcement and an equivalent authenticated broker transport are implemented.
