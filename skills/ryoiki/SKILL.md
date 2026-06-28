---
name: ryoiki
description: Helps coding agents use the ryoiki Jujutsu workspace management CLI safely, including expand/forge/list/enter/collapse/sync/manifest workflows, hooks, shell integration awareness, and verification.
license: MIT
---

# ryoiki

Use `ryoiki` to manage Jujutsu (`jj`) workspaces when a repository relies on it or when the user asks for `ryoiki`, workspace automation, hook-based workspace setup, zoxide registration, fzf selection, or tmux workspace opening.

## When to use

- Prefer `ryoiki` over raw `jj workspace` commands when `.ryoiki.toml` exists, the user mentions hooks/automation, or the task benefits from `ryoiki` integrations (fzf, tmux, zoxide).
- Use raw `jj` only for read-only inspection or when `ryoiki` is unavailable.
- Before changing workspaces, confirm the current directory is inside the intended `jj` repository.

## Terminology

| ryoiki term | Meaning |
|---|---|
| domain (領域) | A jj workspace |
| expand (展開) | Create a new workspace and navigate to it |
| forge (鍛造) | Create a workspace without navigating |
| enter (侵入) | Navigate to an existing workspace |
| collapse (崩壊) | Delete a workspace |
| cursed energy (呪力) | The hook system |

## Safety

- Treat `ryoiki collapse` as destructive. Confirm the exact target before removing a workspace unless the user explicitly named it.
- Do not collapse the current workspace. Switch to another domain first with `ryoiki enter <name>`.
- Avoid interactive fzf flows in unattended automation. Pass explicit workspace names.
- `ryoiki expand` and `ryoiki enter` require shell integration to change the working directory. Without it, use `ryoiki forge` for creation and note that `enter` only prints the path.
- `ryoiki collapse` permanently deletes the workspace directory unless `--keep-dir` is passed.

## Core commands

Run `ryoiki --help` or `ryoiki <command> --help` when command details are uncertain.

- List all workspaces:
  ```bash
  ryoiki list
  ryoiki list --format json
  ryoiki list --format porcelain   # tab-separated, script-friendly
  ryoiki ls --all                  # include stale workspaces
  ```
- Create and navigate (requires shell integration):
  ```bash
  ryoiki expand feature-login
  ryoiki expand hotfix -r main
  ryoiki expand feature-login ~/repos/myapp-login
  ryoiki add feature-login         # alias
  ```
- Create without navigation:
  ```bash
  ryoiki forge ci-check
  ryoiki create ci-check           # alias
  ```
- Navigate to a workspace (requires shell integration):
  ```bash
  ryoiki enter feature-login
  ryoiki enter                     # fzf selector
  ryoiki cd feature-login          # alias
  ```
- Delete a workspace:
  ```bash
  ryoiki collapse feature-login
  ryoiki collapse --force feature-login   # skip confirmation
  ryoiki collapse --keep-dir old-exp      # jj forget only, keep directory
  ryoiki rm feature-login                 # alias
  ```
- Re-apply hooks to existing workspaces:
  ```bash
  ryoiki sync
  ryoiki sync feature-login hotfix
  ryoiki sync --dry-run
  ryoiki sync --hook post-expand
  ```
- Open workspaces in a tmux session:
  ```bash
  ryoiki manifest
  ryoiki manifest feature-login hotfix
  ryoiki manifest --session myapp --window-per-domain
  ryoiki open                      # alias
  ```
- Rename a workspace:
  ```bash
  ryoiki rename feature-login auth-login
  ```
- Show current workspace status:
  ```bash
  ryoiki status
  ryoiki st                        # alias
  ```
- Generate shell integration:
  ```bash
  eval "$(ryoiki init zsh)"
  ryoiki init fish | source
  ryoiki init bash
  ryoiki init nushell
  ```

## Configuration awareness

- Repository-local config lives at `.ryoiki.toml` in the `jj` repository root.
- Global config is at `$XDG_CONFIG_HOME/ryoiki/config.toml` (default: `~/.config/ryoiki/config.toml`).
- The `$RYOIKI_CONFIG` environment variable overrides the global config path.
- `--config <PATH>` CLI flag takes highest precedence.
- Repository-local config merges with (does not override) global config.
- Key config sections: `[core]` (base_dir, dir_format), `[fzf]`, `[tmux]`, `[zoxide]`, `[hooks]`.

## Hook system (呪力)

Hooks are executables run before/after workspace operations. Available hooks:

| Hook | Timing |
|---|---|
| `pre-expand` | Before `expand`/`forge` |
| `post-expand` | After `expand`/`forge` |
| `pre-enter` | Before `enter` |
| `post-enter` | After `enter` |
| `pre-collapse` | Before `collapse` |
| `post-collapse` | After `collapse` |
| `pre-sync` | Before `sync` |
| `post-sync` | After `sync` |

Discovery order: `.ryoiki/hooks/<hook-name>` (repo-local) → `~/.config/ryoiki/hooks/<hook-name>` (global). Both run when both exist (local first).

Environment variables injected into hooks: `RYOIKI_HOOK_NAME`, `RYOIKI_WORKSPACE_NAME`, `RYOIKI_WORKSPACE_PATH`, `RYOIKI_REPO_ROOT`, `RYOIKI_REPO_NAME`, `RYOIKI_CURRENT_WORKSPACE`, `RYOIKI_JJ_CHANGE_ID`.

## Shell integration protocol

`ryoiki expand` and `ryoiki enter` cannot change the parent shell directory directly. When `--shell-output` is passed (done automatically by the wrapper function), they emit:

```
RYOIKI:CD:/absolute/path/to/workspace
```

The shell wrapper function intercepts this line and runs `builtin cd`. Do not parse this output manually in scripts; use `ryoiki list --format porcelain` instead.

## Verification

After creating, removing, or syncing workspaces, verify observable state:

```bash
ryoiki list
ryoiki list --format porcelain
jj workspace list
```

For creation: check the new domain appears and its path exists. For removal: check the domain no longer appears in `ryoiki list` and the directory is gone (unless `--keep-dir` was used).

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error (bad args, workspace not found) |
| 2 | `jj` command failure |
| 3 | Hook failure (when `on_failure = "abort"`) |
| 4 | External tool missing (tmux/fzf required but not installed) |
| 130 | Cancelled by user (Ctrl+C or fzf cancel) |
