# ryoiki (領域) — CLI 仕様書

> Jujutsu VCS のワークスペースを管理する CLI ツール。
> [ofsht](https://github.com/wadackel/ofsht) をリスペクトし、呪術廻戦の「領域展開」をテーマにした設計。

---

## Overview

`ryoiki` (領域) は Jujutsu (`jj`) のワークスペース操作を自動化・簡略化するための CLI ツールです。

`jj workspace add / list / forget` などの基本コマンドをラップしつつ、フック、シェル統合、fzf / tmux / zoxide との連携によって複数ワークスペースを快適に扱えるワークフローを提供します。

### テーマ

| ryoiki 用語 | 意味 |
|---|---|
| 領域 (domain) | jj ワークスペースそのもの |
| 展開 (expand) | ワークスペースの新規作成 |
| 侵入 (enter) | ワークスペースへの移動 |
| 崩壊 (collapse) | ワークスペースの削除 |
| 呪力 (cursed energy) | フックシステム |

---

## Global Flags

すべてのサブコマンドに共通するフラグ。

| Flag | Short | Description |
|---|---|---|
| `--help` | `-h` | ヘルプを表示 |
| `--version` | `-V` | バージョンを表示 |
| `--no-color` | | カラー出力を無効化 |
| `--quiet` | `-q` | 主要出力以外を抑制 |
| `--verbose` | `-v` | 詳細ログを出力 |
| `--config <PATH>` | `-C` | 設定ファイルのパスを明示指定 |
| `--repository <PATH>` | `-R` | jj リポジトリのパスを指定 (デフォルト: 自動検出) |

内部フラグ `--shell-output` はシェル統合関数から使用される (直接の使用は想定しない)。

---

## Commands

### `ryoiki expand` (alias: `add`)

ワークスペースを**新規作成して移動**する。シェル統合が必要。

```
ryoiki expand [OPTIONS] <NAME> [DESTINATION]
```

**Arguments:**

| 引数 | 必須 | Description |
|---|---|---|
| `<NAME>` | Yes | ワークスペース名 |
| `[DESTINATION]` | No | 作成先パス。省略時は `<base_dir>/<dir_format>` |

**Options:**

| Flag | Short | Default | Description |
|---|---|---|---|
| `--revision <REVSET>` | `-r` | `@-` | 作業コピーコミットの親リビジョン |
| `--message <MSG>` | `-m` | | 初期コミットのメッセージ |
| `--sparse <MODE>` | `-s` | `copy` | スパースパターン: `copy` / `full` / `empty` |
| `--no-cd` | | false | 移動せず作成のみ (`forge` と同等) |
| `--no-hooks` | | false | フックを実行しない |
| `--base-dir <PATH>` | | config 参照 | 作成先の基底ディレクトリを上書き |

**Examples:**

```bash
ryoiki expand feature-login
ryoiki expand hotfix -r main
ryoiki expand feature-login ~/repos/myapp-login
ryoiki expand --sparse empty docs-only
```

**Behavior:**

1. `pre-expand` フックを実行
2. `jj workspace add --name <NAME> [--revision <REV>] [DESTINATION]` を実行
3. zoxide が有効なら `zoxide add <DESTINATION>` を実行
4. `post-expand` フックを実行
5. シェル統合を通じて `<DESTINATION>` に `cd`

---

### `ryoiki forge` (alias: `create`)

ワークスペースを**作成するが移動しない**。シェル統合不要。

```
ryoiki forge [OPTIONS] <NAME> [DESTINATION]
```

`expand` と同一オプション (`--no-cd` を除く)。

**Examples:**

```bash
ryoiki forge ci-check
ryoiki forge -R ../other-repo staging
```

---

### `ryoiki list` (alias: `ls`)

すべてのワークスペースを一覧表示する。

```
ryoiki list [OPTIONS]
```

**Options:**

| Flag | Short | Default | Description |
|---|---|---|---|
| `--format <FMT>` | `-f` | `table` | 出力形式: `table` / `simple` / `json` / `porcelain` |
| `--all` | `-a` | false | stale なワークスペースも表示 |
| `--current` | `-c` | false | 現在のワークスペース情報のみ表示 |

**table 出力例:**

```
 DOMAIN           PATH                          CHANGE        DESCRIPTION        STATUS
 main (current)   ~/repos/myapp                 zqupnwos      (empty)            active
 feature-login    ~/repos/myapp-feature-login   kkltznpq      feat: add login    active
 hotfix           ~/repos/myapp-hotfix          (stale)                          stale
```

**json 出力例:**

```json
[
  {
    "name": "main",
    "path": "/home/user/repos/myapp",
    "change_id": "zqupnwos",
    "description": "(empty)",
    "is_current": true,
    "is_stale": false
  }
]
```

**porcelain 出力例 (タブ区切り、スクリプト向け):**

```
main	/home/user/repos/myapp	current
feature-login	/home/user/repos/myapp-feature-login
```

---

### `ryoiki enter` (alias: `cd`)

ワークスペースに移動する。シェル統合が必要。

```
ryoiki enter [OPTIONS] [NAME]
```

**Arguments:**

| 引数 | 必須 | Description |
|---|---|---|
| `[NAME]` | No | 移動先ワークスペース名。省略時は fzf セレクタを起動 |

**Options:**

| Flag | Default | Description |
|---|---|---|
| `--no-fzf` | false | fzf を使わず NAME 引数を必須とする |

**Examples:**

```bash
ryoiki enter feature-login   # 名前指定
ryoiki enter                 # fzf で選択
ryoiki enter feat            # プレフィックス一致 (曖昧な場合は fzf 起動)
```

**fzf プレビュー:**

```
feature-login   /home/user/repos/myapp-feature-login
──────────────────────────────────────────────────────
Change: kkltznpq | Description: feat: add login form

Modified (3):
  M src/auth/login.rs
  M src/ui/login.html
  A tests/auth_test.rs
```

**Behavior:**

1. `pre-enter` フックを実行
2. 対象ワークスペースのパスを解決
3. zoxide が有効なら `zoxide add <PATH>` を実行
4. シェル統合を通じて `cd <PATH>`
5. `post-enter` フックを実行

---

### `ryoiki collapse` (alias: `rm`)

ワークスペースを削除する。

```
ryoiki collapse [OPTIONS] [NAME...]
```

**Arguments:**

| 引数 | 必須 | Description |
|---|---|---|
| `[NAME...]` | No | 削除するワークスペース名。省略時は fzf セレクタ (複数選択可) |

**Options:**

| Flag | Short | Default | Description |
|---|---|---|---|
| `--force` | `-f` | false | 確認プロンプトをスキップ |
| `--keep-dir` | | false | ディレクトリをディスクに残す (`jj workspace forget` のみ実行) |
| `--no-fzf` | | false | fzf を使わない |

**Examples:**

```bash
ryoiki collapse feature-login
ryoiki collapse feature-login hotfix
ryoiki collapse --force feature-login
ryoiki collapse --keep-dir old-experiment
ryoiki collapse            # fzf で複数選択
```

**Behavior:**

1. 確認プロンプト表示 (`--force` がない場合): `Collapse domain "feature-login"? [y/N]`
2. `pre-collapse` フックを実行
3. `jj workspace forget <NAME>` を実行
4. `--keep-dir` がない場合、ディレクトリをディスクから削除
5. zoxide が有効なら `zoxide remove <PATH>` を試みる (失敗は警告扱い)
6. `post-collapse` フックを実行

---

### `ryoiki sync`

既存のすべてのワークスペースに対してフックを再適用する。

```
ryoiki sync [OPTIONS] [NAME...]
```

**Arguments:**

| 引数 | 必須 | Description |
|---|---|---|
| `[NAME...]` | No | 対象ワークスペース名。省略時は全ワークスペースが対象 |

**Options:**

| Flag | Short | Default | Description |
|---|---|---|---|
| `--hook <HOOK>` | | 全フック | 実行するフック名を指定 |
| `--dry-run` | `-n` | false | 実際には実行せず、何を行うかのみ表示 |
| `--parallel` | `-p` | false | ワークスペースを並行処理 |

**Examples:**

```bash
ryoiki sync
ryoiki sync feature-login hotfix
ryoiki sync --dry-run
ryoiki sync --hook post-expand
```

---

### `ryoiki manifest` (alias: `open`)

複数のワークスペースを tmux セッションとして展開する。

```
ryoiki manifest [OPTIONS] [NAME...]
```

**Arguments:**

| 引数 | 必須 | Description |
|---|---|---|
| `[NAME...]` | No | 展開するワークスペース名。省略時は全ワークスペースが対象 |

**Options:**

| Flag | Short | Default | Description |
|---|---|---|---|
| `--session <NAME>` | `-s` | リポジトリ名 | tmux セッション名 |
| `--layout <LAYOUT>` | `-l` | `even-horizontal` | tmux レイアウト |
| `--window-per-domain` | `-w` | false | ワークスペースごとに tmux window を作成 |
| `--no-attach` | | false | アタッチせずにセッションだけ作成 |

**Examples:**

```bash
ryoiki manifest
ryoiki manifest feature-login hotfix
ryoiki manifest --session myapp --window-per-domain
```

**Behavior:**

1. tmux が利用可能か確認 (未インストールならエラー、exit code 4)
2. 指定セッションが既存かチェック (既存なら `tmux attach-session`)
3. `tmux new-session -d -s <SESSION>` で新規セッション作成
4. `--window-per-domain`: ワークスペースごとに `tmux new-window -n <NAME> -c <PATH>`
5. その他: `tmux split-window` でペイン分割し `--layout` を適用
6. `--no-attach` がない場合 `tmux attach-session`

---

### `ryoiki rename`

ワークスペース名を変更する。

```
ryoiki rename <OLD_NAME> <NEW_NAME>
```

**Examples:**

```bash
ryoiki rename feature-login auth-login
```

**Behavior:** `jj workspace rename <OLD_NAME> <NEW_NAME>` を呼び出す。

---

### `ryoiki status` (alias: `st`)

現在のワークスペース情報とすべてのワークスペースのサマリーを表示する。

```
ryoiki status [OPTIONS]
```

**出力例:**

```
Current domain: feature-login
Path:           ~/repos/myapp-feature-login
Change ID:      kkltznpq
Description:    feat: add login form

Other domains (3):
  main          ~/repos/myapp                active
  hotfix        ~/repos/myapp-hotfix         active
  docs          ~/repos/myapp-docs           stale
```

---

### `ryoiki init`

シェル統合スクリプトを標準出力に出力する。

```
ryoiki init [OPTIONS] <SHELL>
```

**Arguments:**

| 引数 | 必須 | Description |
|---|---|---|
| `<SHELL>` | Yes | `bash` / `zsh` / `fish` / `nushell` |

**Options:**

| Flag | Default | Description |
|---|---|---|
| `--hook-key <KEY>` | なし | `enter` コマンドにバインドするキー (例: `\C-g`) |

**Examples:**

```bash
# ~/.bashrc / ~/.zshrc に追記
eval "$(ryoiki init zsh)"

# fish
ryoiki init fish | source

# キーバインドも設定
eval "$(ryoiki init zsh --hook-key '\C-g')"
```

---

## Configuration

### ファイル優先順位 (高い順)

1. `--config <PATH>` フラグで明示指定されたファイル
2. `<repo-root>/.ryoiki.toml` (リポジトリローカル設定)
3. `$RYOIKI_CONFIG` 環境変数で指定されたパス
4. `$XDG_CONFIG_HOME/ryoiki/config.toml` (デフォルト: `~/.config/ryoiki/config.toml`)

リポジトリローカル設定はグローバル設定を**マージ**する (上書きではなく)。

### `~/.config/ryoiki/config.toml`

```toml
[core]
# ワークスペースを作成するデフォルトの基底ディレクトリ
# 相対パスは jj リポジトリルートからの相対パスとして解釈される
base_dir = "~/repos"

# ディレクトリ名フォーマット。変数: {name}, {repo}, {date}
dir_format = "{repo}-{name}"

# enter/expand 後にワークスペースのステータスを表示するか
show_status_on_enter = true

# カラー出力: "auto" | "always" | "never"
color = "auto"

[list]
# デフォルト出力形式: "table" | "simple" | "json" | "porcelain"
format = "table"

# stale なワークスペースをデフォルトで表示するか
show_stale = false

# テーブルの列 (表示する列と順序)
columns = ["name", "path", "change_id", "description", "status"]

[fzf]
# fzf が利用可能な場合に使用するか
enabled = true

# fzf に渡す追加オプション
opts = "--height=40% --border --reverse"

# fzf プレビューウィンドウ
preview = true
preview_window = "right:50%:wrap"

[tmux]
# manifest コマンドのデフォルトレイアウト
default_layout = "even-horizontal"

# セッション名フォーマット。変数: {repo}
session_format = "{repo}"

# 既存セッションへの自動アタッチ
auto_attach = true

[zoxide]
# enter/expand 時に zoxide に自動登録するか
enabled = true

[hooks]
# フックの有効/無効
enabled = true

# フックスクリプトの置き場 (省略時: "$XDG_CONFIG_HOME/ryoiki/hooks/")
hook_dir = "~/.config/ryoiki/hooks"

# フックのタイムアウト秒数
timeout_seconds = 30

# フックが失敗した場合の動作: "abort" | "warn" | "ignore"
on_failure = "warn"
```

### `.ryoiki.toml` (リポジトリローカル設定)

```toml
[core]
base_dir = "./workspaces"  # リポジトリルートからの相対パス
dir_format = "{name}"

[hooks]
hook_dir = ".ryoiki/hooks"

# フックに注入される追加環境変数
[hooks.env]
NODE_ENV = "development"
DATABASE_URL = "postgres://localhost/myapp_dev"
```

---

## Hook System (呪力システム)

フックは特定のワークスペース操作の前後に実行される実行可能ファイル。

### フック一覧

| フック名 | タイミング | 用途例 |
|---|---|---|
| `pre-expand` | `expand`/`forge` 実行前 | バリデーション、事前チェック |
| `post-expand` | `expand`/`forge` 実行後 | 依存関係インストール、エディタ起動 |
| `pre-enter` | `enter` 実行前 | 現在のワークスペースの保存処理 |
| `post-enter` | `enter` 実行後 | 環境変数の設定、プロンプト更新 |
| `pre-collapse` | `collapse` 実行前 | 未保存変更の確認 |
| `post-collapse` | `collapse` 実行後 | クリーンアップ処理 |
| `pre-sync` | `sync` 実行前 | |
| `post-sync` | `sync` 実行後 | |

### ディスカバリー順序

1. リポジトリローカルフックディレクトリ (`.ryoiki/hooks/<hook-name>`)
2. グローバルフックディレクトリ (`~/.config/ryoiki/hooks/<hook-name>`)

両方に同名のフックが存在する場合、**両方が実行される** (ローカル → グローバルの順)。

### 実行時環境変数

| 変数 | Description |
|---|---|
| `RYOIKI_HOOK_NAME` | 実行中のフック名 |
| `RYOIKI_WORKSPACE_NAME` | 対象ワークスペース名 |
| `RYOIKI_WORKSPACE_PATH` | ワークスペースの絶対パス |
| `RYOIKI_REPO_ROOT` | jj リポジトリのルート絶対パス |
| `RYOIKI_REPO_NAME` | リポジトリ名 |
| `RYOIKI_CURRENT_WORKSPACE` | 操作前の現在ワークスペース名 |
| `RYOIKI_JJ_CHANGE_ID` | ワークスペースの Change ID (利用可能な場合) |

### フックスクリプト例

**`post-expand` — 依存関係の自動インストール:**

```bash
#!/usr/bin/env bash
set -euo pipefail

cd "${RYOIKI_WORKSPACE_PATH}"

if [[ -f "package.json" ]]; then
  npm install --silent
fi

if [[ -f "Cargo.toml" ]]; then
  cargo fetch --quiet
fi
```

**`pre-collapse` — 未保存変更のチェック:**

```bash
#!/usr/bin/env bash
set -euo pipefail

cd "${RYOIKI_WORKSPACE_PATH}"

if ! jj diff --quiet 2>/dev/null; then
  echo "Warning: Domain '${RYOIKI_WORKSPACE_NAME}' has uncommitted changes."
  read -r -p "Proceed with collapse? [y/N] " answer
  [[ "${answer}" =~ ^[Yy]$ ]] || exit 1
fi
```

---

## Shell Integration

### 設計原則

子プロセス (ryoiki バイナリ) から親シェルのディレクトリを変更することはできない。そのため:

- ryoiki は `--shell-output` フラグが渡されると、制御シーケンス `RYOIKI:CD:<path>` を stdout に出力する
- シェル関数がこれをパースして `builtin cd` を実行する

### IPC プロトコル

```
RYOIKI:CD:/home/user/repos/myapp-feature-login
```

制御シーケンスは stdout の末尾に出力される。シェル関数は `RYOIKI:` プレフィックスの行を除いた内容をユーザーに表示する。

### Bash/Zsh 統合

```zsh
# ~/.zshrc
eval "$(ryoiki init zsh)"
```

`ryoiki init zsh` が生成するシェル関数:

```zsh
ryoiki() {
  local _output _exit _cd_path
  _output="$(command ryoiki --shell-output "$@" 2>&1)"
  _exit=$?
  _cd_path="$(printf '%s\n' "$_output" | grep '^RYOIKI:CD:' | tail -1 | cut -d: -f3-)"
  printf '%s\n' "$_output" | grep -v '^RYOIKI:'
  if [[ $_exit -ne 0 ]]; then
    return $_exit
  fi
  if [[ -n "$_cd_path" ]]; then
    builtin cd "$_cd_path"
  fi
}

# 現在のドメイン名を取得するユーティリティ関数
ryoiki_current_domain() {
  command ryoiki list --format porcelain 2>/dev/null \
    | awk -F'\t' '$3=="current"{print $1}'
}
```

### Fish 統合

```fish
# ~/.config/fish/config.fish
ryoiki init fish | source
```

### Nushell 統合

```nu
# config.nu
ryoiki init nushell | save ~/.config/ryoiki/integration.nu
source ~/.config/ryoiki/integration.nu
```

### Starship プロンプト統合例

```toml
# ~/.config/starship.toml
[custom.ryoiki]
command = "ryoiki list --format porcelain | awk -F'\\t' '$3==\"current\"{print $1}'"
when = "jj root 2>/dev/null"
symbol = "領 "
style = "bold cyan"
format = "[$symbol$output]($style) "
```

---

## Output Design

### カラースキーム

| 要素 | 色 |
|---|---|
| 現在のワークスペース | Cyan (Bold) |
| アクティブなワークスペース | White |
| Stale なワークスペース | Yellow |
| エラーメッセージ | Red |
| 成功メッセージ | Green |
| Change ID | Magenta |
| パス | Blue (Dimmed) |
| フックメッセージ | Dim |

### エラーメッセージ設計

JJK テーマでエラーメッセージを構成する:

```
error: Domain expansion failed — workspace "feature-login" already exists.
  hint: Use `ryoiki list` to see existing domains.
```

```
error: Cannot collapse current domain "main".
  hint: Enter another domain first with `ryoiki enter <name>`.
```

```
warning: Domain "hotfix" is stale (working copy not updated).
  hint: Run `jj workspace update-stale` in that directory to recover.
```

---

## Integrations

### fzf

`enter` と `collapse` で NAME 引数を省略した場合に自動起動する。`[fzf]` セクションでカスタマイズ可能。

### tmux

`manifest` コマンドが管理する。tmux が未インストールの場合はエラー (exit code 4)。

### zoxide

`expand` と `enter` 実行時に自動的に `zoxide add <path>` を呼び出す。`collapse` 実行時は `zoxide remove <path>` を試みる (失敗は警告扱い)。

---

## Exit Codes

| コード | 意味 |
|---|---|
| 0 | 成功 |
| 1 | 一般エラー (引数不正、ワークスペース未発見など) |
| 2 | jj コマンド失敗 |
| 3 | フック失敗 (`on_failure = "abort"` の場合) |
| 4 | 外部ツール未発見 (tmux/fzf が必要なのに未インストール) |
| 130 | ユーザーによるキャンセル (Ctrl+C / fzf でのキャンセル) |

---

## Crate 選定方針

| 用途 | クレート |
|---|---|
| CLI パース | `clap` v4 (derive マクロ) |
| 設定ファイル | `toml` + `serde` |
| カラー出力 | `owo-colors` |
| エラーハンドリング | `anyhow` + `thiserror` |
| テーブル表示 | `comfy-table` |
| JSON 出力 | `serde_json` |
| プロセス実行 | `std::process::Command` |
| fzf 連携 | 外部 `fzf` コマンド呼び出し (skim より安定) |
| ファイルパス操作 | `std::path` + `dirs` (XDG ディレクトリ) |

---

## Command Summary

| コマンド | エイリアス | シェル統合要否 | jj primitive |
|---|---|---|---|
| `expand` | `add` | 必要 | `workspace add` |
| `forge` | `create` | 不要 | `workspace add` |
| `list` | `ls` | 不要 | `workspace list` |
| `enter` | `cd` | 必要 | `workspace root` |
| `collapse` | `rm` | 不要 | `workspace forget` + rm |
| `sync` | | 不要 | — (フック再実行) |
| `manifest` | `open` | 不要 | `workspace list` + tmux |
| `rename` | | 不要 | `workspace rename` |
| `status` | `st` | 不要 | `workspace list` |
| `init` | | N/A | — (シェル統合生成) |
