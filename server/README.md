ローカルの開発環境をセットアップするために、下記の操作を一度実施する必要がある。

DBにスキーマとテーブルをセットアップ
```
export DATABASE_URL=postgres://postgres:example@db/ccs_db
sea-orm-cli migrate up
```

sample.envファイルを.envへリネームし、環境にあった変数を設定する
