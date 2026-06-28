# ryoiki (領域)

> Jujutsu (jj) のワークスペースを管理する CLI ツール。「領域展開」をテーマにした設計。

[![CI](https://github.com/UtakataKyosui/ryoiki/actions/workflows/ci.yml/badge.svg)](https://github.com/UtakataKyosui/ryoiki/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ryoiki.svg)](https://crates.io/crates/ryoiki)

## テーマ

| ryoiki 用語 | 意味 |
|---|---|
| 領域 (domain) | jj ワークスペースそのもの |
| 展開 (expand) | ワークスペースの新規作成 |
| 侵入 (enter) | ワークスペースへの移動 |
| 崩壊 (collapse) | ワークスペースの削除 |
| 呪力 (cursed energy) | フックシステム |

## インストール

```bash
cargo install ryoiki
```

## AI エージェント向け AgentSkill

`ryoiki` は AI コーディングエージェントが安全にワークスペースを操作するための AgentSkill を提供します。GitHub CLI の `gh skill` コマンドでインストールできます。

```bash
# スキルをインストール
gh skill install UtakataKyosui/ryoiki ryoiki --agent codex --scope project

# インストール前にプレビュー
gh skill preview UtakataKyosui/ryoiki ryoiki

# スキルを更新
gh skill update ryoiki
```

> [!NOTE]
> `gh skill` は GitHub CLI のプレビュー機能です。AgentSkill はエージェントに `ryoiki` CLI の使い方を教えるものであり、`ryoiki` バイナリ自体はインストールしません。

## シェル統合

```bash
# ~/.zshrc
eval "$(ryoiki init zsh)"

# ~/.bashrc
eval "$(ryoiki init bash)"

# fish
ryoiki init fish | source
```

## コマンド一覧

| コマンド | エイリアス | 説明 |
|---|---|---|
| `expand` | `add` | ワークスペースを作成して移動 |
| `forge` | `create` | ワークスペースを作成 (移動しない) |
| `list` | `ls` | ワークスペース一覧 |
| `enter` | `cd` | ワークスペースに移動 |
| `collapse` | `rm` | ワークスペースを削除 |
| `sync` | | フックを全ワークスペースに再適用 |
| `manifest` | `open` | tmux セッションとして展開 |
| `rename` | | ワークスペース名を変更 |
| `status` | `st` | 現在のワークスペース情報を表示 |
| `init` | | シェル統合スクリプトを出力 |

詳細は [SPEC.md](./SPEC.md) を参照してください。

## ライセンス

MIT — 詳細は [LICENSE](./LICENSE) を参照。
