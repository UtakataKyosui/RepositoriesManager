# Repositories Manager

GitHubリポジトリの名前、説明、タグ（topics）を管理するためのTUI（Terminal User Interface）アプリケーションです。

## 特徴

- 表形式で見やすいリポジトリ一覧表示
- リポジトリの説明（description）の編集
- リポジトリのトピック（topics）の編集
- GitHub CLIを内部で使用するため、シンプルで安全

## 前提条件

- Rust (1.70以上)
- GitHub CLI (`gh`) がインストールされ、認証済みであること

GitHub CLIのインストール:
```bash
# Ubuntuの場合
sudo apt install gh

# 認証
gh auth login
```

## インストール

```bash
# プロジェクトをクローン
git clone <repository-url>
cd RepositoriesManager

# ビルド
cargo build --release

# 実行
./target/release/repositories-manager
```

## 使い方

アプリケーションを起動すると、自動的にあなたのGitHubリポジトリ一覧が表示されます。

### キーボード操作

#### 通常モード
- `↑` / `k`: 上の行に移動
- `↓` / `j`: 下の行に移動
- `d`: 選択したリポジトリの説明（Description）を編集
- `t`: 選択したリポジトリのトピック（Topics）を編集
- `q` または `Ctrl+C`: アプリケーションを終了

#### 編集モード
- `Enter`: 変更を保存してGitHubに反映
- `Esc`: 編集をキャンセル
- `Backspace`: 1文字削除
- その他のキー: 文字入力

### トピックの編集

トピックはカンマ区切りで入力します:
```
rust, cli, terminal, github
```

空欄にすると、すべてのトピックが削除されます。

## 技術スタック

- **Rust**: プログラミング言語
- **Ratatui**: TUIフレームワーク
- **Crossterm**: ターミナル操作ライブラリ
- **GitHub CLI**: リポジトリ情報の取得・更新

## ライセンス

MIT License

## 貢献

プルリクエストやイシューの報告を歓迎します！
