# TODOリスト
定期処理で対応する予定の処理を実装する
- 期限切れのtemp_accountレコードの削除
- 期限切れのpwd_change_reqレコードの削除

開発環境用のDockerfileについて以下の考慮を行い、整理する
- ステージの考え方（ビルド用イメージ、デプロイ用イメージ）を使ったDockerfileとする
- フロントエンド、バックエンドでDockerfileを分ける

Vueのデバッグ（ブレークポイントを貼って、そこで止めること）ができなくなった問題の解決

Veturの[問題](https://github.com/vuejs/vetur/issues/3323)解決後、typescriptのdependencyを~4.5.5に戻す (372657113b7dbcc4661b2ebe28490dcb2cc8a674をrevert)

APIサーバのRate Limitについて検討、実装する（ELB界面で実施するのか、それともAPIサーバ界面で実装するのか検討）
(APIサーバで実装する際の参考: https://github.com/tokio-rs/axum/issues/278)

SDKが実装時点でVirtual Hosted-Styleを[サポートしていない](https://github.com/awslabs/aws-sdk-rust/discussions/485)。そのため、それまでPath-Styleで実装し、Virtual Hosted-Styleがサポートされた後、修正する

# NOTE
## 開発環境
開発環境にはVS Code (IDE) とRemote Container (VS Code用拡張プラグイン) が必須となる。それらを用意し開発を行う。
### VS Code (IDE) とRemote Container (VS Code用拡張プラグイン) が必須の理由
ファイルストレージとしてAWS S3との連携を想定している。S3は、バケットにそのバケット名を含むホスト名を割り当てる。開発者はバケットにアクセスするため、そのバケット名が含まれたホスト名を用いてそのバケットへアクセスする（virtual-hosted style）バケットが複数ある場合、どのバケットにアクセスすべきか決めるため、ローカルの開発環境でもホスト名の利用が必須となる。ホスト名を利用することは同時に名前解決システムの導入も必須となる。名前解決システムはDockerのネットワーク上に自動構築されるDNSサーバを利用するのが最も簡単であると判断し、それを利用することとする。開発中のアプリが名前解決ができるようにするには、そのアプリをDockerネットワーク上のコンテナ内で起動し、動作確認する必要がある。コンテナ内でアプリのビルドと動作確認が可能な環境が、現状VS CodeとRemote Containerだけのため、開発環境にこれらが必須となる。

# TERMINOLOGY
## ccs
Career Change Supporterの略称
