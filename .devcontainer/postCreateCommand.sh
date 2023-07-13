#!/bin/bash -eu

# 開発に役立つツールのインストール
cargo install --locked ripgrep && \
  cargo install --locked fd-find && \
  cargo install --locked cargo-sort

# 以下バックエンドの環境の準備
pushd server > /dev/null

while ! echo exit | curl -s telnet://db:5432;
do
  echo "waiting for db to launch"
  sleep 1
done
echo "db launched"

echo "db initialization start"

# ロールの追加、DBインスタンスの作成を実施
export DB_HOST=db
export DB_PORT=5432
export DB_ADMIN_USER=postgres
export DB_ADMIN_PASSWORD=example
export USER_APP_PASSWORD=test1234
export ADMIN_APP_PASSWORD=test13579
./files_for_compose/initdb/init.sh > /dev/null;

# DBインスタンスにスキーマとテーブルを作成
export DATABASE_URL=postgres://postgres:example@db/ccs_db
sea-orm-cli migrate up > /dev/null;

echo "db initialization finish"

while ! curl -I -s http://opensearch:9200 > /dev/null;
do
  echo "waiting for opensearch to launch"
  sleep 1
done
echo "opensearch launched"

echo "opensearch initialization start"
# インデックスの生成
curl -s -XPUT -H "Content-Type: application/json" --data "@files_for_compose/opensearch/index_definition/index.json" "http://opensearch:9200/users" > /dev/null
# replicaシャードの数を0に設定（開発環境の設定であり、本番環境では実施しない設定）
# 開発環境では、OpenSearchは単一ノードで構成する。単一ノードの場合、replicaシャードを配置するための別ノードが存在しない。
# そのため、それに起因してインデックスのステータスがyellowとなる。開発環境においては、replicaシャードが存在しないことは問題とならない。
# したがって、このステータスをgreenにしておくため、インデックスに対して下記のコマンドを打ってレプリカの数を0に設定しておく。
curl -s -XPUT -H "Content-Type: application/json" -d '{ "index": { "number_of_replicas": 0 } }' "http://opensearch:9200/users/_settings" > /dev/null
echo "opensearch initialization end"

popd > /dev/null
