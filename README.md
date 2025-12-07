# Repositories Manager

GitHubリポジトリの名前、説明、タグ（topics）を管理するためのTUI（Terminal User Interface）アプリケーションです。

## 特徴

- 表形式で見やすいリポジトリ一覧表示
- リポジトリの検索（名前、説明、タグで検索可能）
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
- `/`: 検索モードに入る
- `Esc`: 検索フィルタをクリア（すべてのリポジトリを表示）
- `d`: 選択したリポジトリの説明（Description）を編集
- `t`: 選択したリポジトリのトピック（Topics）を編集
- `q` または `Ctrl+C`: アプリケーションを終了

#### 検索モード
- テキスト入力: 検索クエリを入力（リポジトリ名、説明、タグで検索）
- `Enter`: 検索を実行
- `Esc`: 検索をキャンセルしてフィルタをクリア
- `Backspace`: 1文字削除

#### 編集モード
- `Enter`: 変更を保存してGitHubに反映
- `Esc`: 編集をキャンセル
- `Backspace`: 1文字削除
- その他のキー: 文字入力

### 検索機能

`/`キーを押すと検索モードに入ります。検索クエリを入力すると、以下の項目で部分一致検索が行われます：
- リポジトリ名
- 説明文
- トピック（タグ）

`Enter`を押すと検索が実行され、一致するリポジトリのみが表示されます。
`Esc`を押すと検索フィルタがクリアされ、すべてのリポジトリが表示されます。

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
