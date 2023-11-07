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

（※）下記の二種類を手動で対応する。
<ol>
  <li>パラメータストアにECSで利用するシークレットを含む環境変数をSecureStringのパラメータとして構築。SecureStringのパラメータは、テンプレート作成時点でCloudFormationに対応していない（作成は完全に未対応、読み込みも制限がある）ため手動で構築する。前述の通り使い勝手が悪いため、基本的に使わない想定でテンプレートを構築していた。しかし、ECSがコンテナを起動するとき、環境変数に平文としてシークレットを埋め込むのはリスクが高いと判断し、その箇所だけシークレットの読み込みをする際にのみ利用する。</li>
  <li>パラメータストアに収納代行用の銀行口座に関連する情報をStringのパラメータとして構築。銀行口座に関連する情報は日本語を含んでおり、それらをCloudFormationテンプレート内で扱うことは困難である（CloudFormationテンプレートはASCII文字以外に対応していない）従って、それらの情報はパラメータストアに保管し、CloudFormationテンプレートからはパラメータストアを介して銀行口座に関連する情報を取得するようにする。</li>
</ol>

## Route53（Hosted Zone）
下記の対応を行う
- CAレコードを作成する
- DNSSECに対応させる（DNSSECに対応させる際、KSKの管理にKMS上にCMKを作ることになる）
- DNSSECを有効にした際に監視すべき指標をCloudWatch Alarmに設定しておく

## Systems Manager
下記のSecureStringパラメータを作成する
TODO

## SES
下記の対応を行う
- 問い合わせを受け付けるためのメールアドレスを用意し、アカウントを作成する。そして、メールの受信ができるようにRoute53にレコードを作成する（メール送信だけでなく、メール受信もできるようにus-east-1に構築する）
- DMARCに対応させる（SPF、DKIMの対応を行う）
- バウンスレートの指標をCloudWatch Alarmに設定しておく

## ECR
TODO

## CI結果を格納するS3バケット
TODO
