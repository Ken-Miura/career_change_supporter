# プロジェクト構成

## client
フロントエンド向けプロジェクトの集合

## infra
インフラを管理、整備するためのコードの集合

## server
バックエンド向けプロジェクトの集合

# TODOリスト
評価数の最大値と最大値を超える場合の処理の仕様の策定と実装

Vueのデバッグ（ブレークポイントを貼って、そこで止めること）ができなくなった問題の解決

クライアント側のコードでオーディオを使うためにgetUserMediaを使う。インターネット上にデプロイした際にその関数を使う際にFeature-Policyが必要確認する。
参考1: https://developer.mozilla.org/ja/docs/Web/API/MediaDevices/getUserMedia
参考2: https://developer.mozilla.org/ja/docs/Web/HTTP/Headers/Permissions-Policy

client/admin_app以下の一部用意していない単体テストコード

スモークテストで下記を確認する
- WAFのAWSマネージドのルールに関して、クエリ、パスのサイズを制限するものに引っかからないか確認する
- 送信するメールの文面全般
- ログイン時にログイン処理が250msかかるコストを見つけておく

# NOTE

## 開発環境
client、serverの開発環境にはVS Code (IDE) とRemote Container (VS Code用拡張プラグイン) を用意し、開発を行う。

### VS Code (IDE) とRemote Container (VS Code用拡張プラグイン) を利用する理由
ファイルストレージとしてAWS S3との連携を想定している。S3は、バケットにそのバケット名を含むホスト名を割り当てる。開発者はバケットにアクセスするため、そのバケット名が含まれたホスト名を用いてそのバケットへアクセスする（virtual-hosted style）バケットが複数ある場合、どのバケットにアクセスすべきか決めるため、ローカルの開発環境でもホスト名の利用が必須となる。ホスト名を利用することは同時に名前解決システムの導入も必須となる。名前解決システムはDockerのネットワーク上に自動構築されるDNSサーバを利用するのが最も簡単であると判断し、それを利用することとする。開発中のアプリが名前解決ができるようにするには、そのアプリをDockerネットワーク上のコンテナ内で起動し、動作確認する必要がある。コンテナ内でアプリのビルドと動作確認が可能な環境が、現状VS CodeとRemote Containerだけのため、開発環境にこれらが必須となる。

# TERMINOLOGY

## ccs
Career Change Supporterの略称
