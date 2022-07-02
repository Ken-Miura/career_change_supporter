# TODOリスト
評価数の最大値と最大値を超える場合の処理の仕様の策定と実装

定期処理で対応する予定の処理を実装する
- 期限切れのtemp_accountレコードの削除
- 期限切れのpwd_change_reqレコードの削除
- deleted_user_accountに紐づくidentity、career、tenant、consulting_fee、create_identity_request、update_identity_request、create_career_requestの削除

開発環境用のDockerfileについて以下の考慮を行い、整理する
- ステージの考え方（ビルド用イメージ、デプロイ用イメージ）を使ったDockerfileとする
- フロントエンド、バックエンドでDockerfileを分ける

Vueのデバッグ（ブレークポイントを貼って、そこで止めること）ができなくなった問題の解決

Veturの[問題](https://github.com/vuejs/vetur/issues/3323)解決後、typescriptのdependencyを~4.5.5に戻す (372657113b7dbcc4661b2ebe28490dcb2cc8a674をrevert)

APIサーバのRate Limitについて検討、実装する（ELB界面で実施するのか、それともAPIサーバ界面で実装するのか検討）<br>
APIサーバで実装する際の参考: https://github.com/tokio-rs/axum/issues/278<br>
上記のURLを参考にするだけではコンパイルエラーとなる。axumはmiddlewareに対してInfallibleなエラーを許していないので下記のようにエラーハンドリングも追加する必要がある。<br>
https://docs.rs/axum/latest/axum/error_handling/index.html#applying-fallible-middleware

実装時点でSDKがVirtual Hosted-Styleを[サポートしていなかった](https://github.com/awslabs/aws-sdk-rust/discussions/485)。そのため、それまでPath-Styleで実装し、Virtual Hosted-Styleがサポートされた後、修正する

AWS内部の通信（ELB→APサーバ、APサーバ→SMTPサーバ、APサーバ→Redis、APサーバ→DB、APサーバ→OpenSearch）にTLSを用いるかどうか検討する

# NOTE
## 開発環境
開発環境にはVS Code (IDE) とRemote Container (VS Code用拡張プラグイン) が必須となる。それらを用意し開発を行う。
### VS Code (IDE) とRemote Container (VS Code用拡張プラグイン) が必須の理由
ファイルストレージとしてAWS S3との連携を想定している。S3は、バケットにそのバケット名を含むホスト名を割り当てる。開発者はバケットにアクセスするため、そのバケット名が含まれたホスト名を用いてそのバケットへアクセスする（virtual-hosted style）バケットが複数ある場合、どのバケットにアクセスすべきか決めるため、ローカルの開発環境でもホスト名の利用が必須となる。ホスト名を利用することは同時に名前解決システムの導入も必須となる。名前解決システムはDockerのネットワーク上に自動構築されるDNSサーバを利用するのが最も簡単であると判断し、それを利用することとする。開発中のアプリが名前解決ができるようにするには、そのアプリをDockerネットワーク上のコンテナ内で起動し、動作確認する必要がある。コンテナ内でアプリのビルドと動作確認が可能な環境が、現状VS CodeとRemote Containerだけのため、開発環境にこれらが必須となる。

### OSの設定値の変更
検索エンジンとしてOpenSearchを利用する。OpenSearchを安定して動作されるため、下記のリンクの設定に従い、vm.max_map_countを262144以上に設定する。<br>
https://opensearch.org/docs/latest/opensearch/install/important-settings/

### インデックスの生成
docker-composeを立ち上げた後、OpenSearchに対して下記のコマンドを打ってインデックスを生成する
```
curl -XPUT -H "Content-Type: application/json" --data "@files_for_docker_compose/opensearch/index_definition/index.json" "http://opensearch:9200/users"
```

### replicaシャードの数を0に設定（開発環境の設定であり、本番環境では実施しない設定）
開発環境では、OpenSearchは単一ノードで構成する。単一ノードの場合、replicaシャードを配置するための別ノードが存在しない。そのため、それに起因してインデックスのステータスがyellowとなる。開発環境においては、replicaシャードが存在しないことは問題とならない。そのため、このステータスをgreenにしておくため、[インデックスの生成](#インデックスの生成)で作成したインデックスに対して、下記のコマンドを打ってレプリカの数を0に設定しておく。
```
curl -XPUT -H "Content-Type: application/json" -d '{ "index": { "number_of_replicas": 0 } }' "http://opensearch:9200/users/_settings"
```

# TERMINOLOGY
## ccs
Career Change Supporterの略称
