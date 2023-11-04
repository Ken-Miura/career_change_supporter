# 構築順
依存関係があるため構築順序の通りに構築する必要がある。
<ol>
  <li>NOTEに記載のリソースを手動で構築</li>
  <li>network.yaml</li>
  <li>static-files.yaml</li>
  <li>data-store.yaml</li>
  <li>load-balancer.yaml</li>
  <li>application-cluster.yaml</li>
  <li>applicationsディレクトリ以下の全てのCloudFormationテンプレート</li>
  <li>request-controller.yaml</li>
</ol>

# 構築リージョン
## ap-northeast-1
- request-controller.yaml以外のCloudFormationテンプレート

## us-east-1
- request-controller.yaml (CloudFrontに紐づくWebACLやACMがus-east-1でしか構築できないため)

# DBのマイグレーション方法
TODO

# APサーバの更新方法
TODO

# インフラリソースのスペックの更新方法
ymal内の該当するリソース（インスタンスタイプ等）を書き換えてCloudFormationにて更新する。更新の際は既存のリソースを削除する動作をしていないか確認するため、必ず変更セットを見て問題ないことがわかってから更新を行う。

# NOTE
Route53（Hosted Zone）、Systems Manager （※）、SES、ECR、CI結果を格納するS3バケットはCloudFormationの管理対象外としている（マネジメントコンソールから手動で構築している）

（※）パラメータストアのSecureStringのパラメータ。SecureStringのパラメータは、テンプレート作成時点でCloudFormationに対応していない（作成は完全に未対応、読み込みも制限がある）ため手動で構築する。前述の通り使い勝手が悪いため、基本的に使わない想定でテンプレートを構築していた。しかし、ECSがコンテナを起動するとき、環境変数に平文としてシークレットを埋め込むのはリスクが高いと判断し、その箇所だけシークレットの読み込みをする際にのみ利用する。

## Route53（Hosted Zone）
下記の対応を行う
- CAレコードを作成する
- DNSSECに対応させる（DNSSECに対応させる際、KSKの管理にKMS上にCMKを作ることになる）

## Systems Manager
下記のSecureStringパラメータを作成する
TODO

## SES
下記の対応を行う
- 問い合わせを受け付けるためのメールアドレスを用意し、アカウントを作成する。そして、メールの受信ができるようにRoute53にレコードを作成する
- DMARCに対応させる（SPF、DKIMの対応を行う）

## ECR
TODO

## CI結果を格納するS3バケット
TODO

# メモ
2023年8月13日現在の状況
テスト用の環境を用意し、最低限の動作確認は完了した。しかし、RDS、OpenSearch等々は中々お金がかかるので一時削除を行う。
削除する前に、テストで動作確認出来た状態に戻せるように設定を保存しておきたかった。従って、既存のリソースをFormer2の機能を試し、テンプレートを作成した。
それをコミットして残しておく。本番用にパラメータ調整した、かつ必要なリソース全体を持っているテンプレートは最後に作成する予定。

上記に記載の一時的に残しているテンプレート
- ecs_service.yml
