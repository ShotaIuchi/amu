# Commit Message Schema

## Format

```
<type>: <subject>

[body]

[footer]
```

## Type (必須)

| Type | 説明 |
|------|------|
| `feat` | 新機能の追加 |
| `fix` | バグ修正 |
| `docs` | ドキュメントのみの変更 |
| `style` | コードの意味に影響しない変更（空白、フォーマット等） |
| `refactor` | バグ修正でも機能追加でもないコード変更 |
| `perf` | パフォーマンス改善 |
| `test` | テストの追加・修正 |
| `chore` | ビルドプロセスや補助ツールの変更 |

## Subject (必須)

- 50文字以内を推奨
- 末尾にピリオドを付けない
- 命令形で記述（日本語の場合は体言止め可）

## Body (任意)

- 変更の理由や背景を記述
- 72文字で改行を推奨

## Footer (任意)

- Breaking changes: `BREAKING CHANGE: <description>`
- Issue参照: `Closes #123`, `Fixes #456`

## 例

```
feat: ユーザー認証機能を追加

OAuth2.0を使用したログイン機能を実装。
Google と GitHub のプロバイダーに対応。

Closes #42
```

```
fix: 検索結果が0件の場合のエラーを修正
```

```
chore: bump version to 0.1.6
```
