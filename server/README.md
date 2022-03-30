# ローカルの開発環境のセットアップ

## DBにスキーマとテーブルを作成
```
export DATABASE_URL=postgres://postgres:example@db/ccs_db
sea-orm-cli migrate up
```

## 環境変数の用意
sample.envファイルを.envへリネームし、環境にあった変数を設定する

# DBのテーブルの変更と反映

## DBのテーブルを変更
migration/src以下のソースコードを変更する

## 変更されたテーブルをentity以下のソースコードに反映
```
export DATABASE_URL=postgres://postgres:example@db/ccs_db
sea-orm-cli generate entity -s ccs_schema -o entity/src
```