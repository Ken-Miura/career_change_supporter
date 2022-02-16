ローカルの開発環境をセットアップするために、下記の操作を一度実施する必要がある。

DBセットアップ
```
cd common
diesel setup --database-url=postgres://postgres:example@db/ccs_db
```

sample.envファイルを.envへリネームし、環境にあった変数を設定する
