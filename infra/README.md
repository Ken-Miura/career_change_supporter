# 構築順
依存関係があるため構築順序の通りに構築する必要がある。
<ol>
  <li>NOTEに記載のリソースを手動で構築</li>
  <li>artifacts-store.yaml</li>
  <li>network.yaml</li>
  <li>static-files.yaml</li>
  <li>data-store.yaml</li>
  <li>load-balancer.yaml</li>
  <li>application-cluster.yaml</li>
  <li>applicationsディレクトリ以下の全てのCloudFormationテンプレート</li>
  <li>request-controller.yaml</li>
  <li>deploy-user.yaml</li>
</ol>

## 初回構築時の注意
下記のCloudFormationテンプレートは構築時にTLS証明書を作成する。初めて構築する際は、TLS証明書でのドメイン検証をWeb UIから実施する必要がある（スタック構築中にUPDATE_IN_PROGRESSの状態でドメイン検証待ちの状態になるので、Web UIからドメイン検証を行う）一度ドメイン検証を実施した後は、次回以降のスタック構築の際にドメイン検証の実施は必要ない（UPDATE_IN_PROGRESSの状態でドメイン検証待ちの状態になることはない）
<ol>
  <li>load-balancer.yaml</li>
  <li>request-controller.yaml</li>
</ol>

# 構築リージョン
## ap-northeast-1
- request-controller.yaml以外のCloudFormationテンプレート

## us-east-1
- request-controller.yaml (CloudFrontに紐づくWebACLやACMがus-east-1でしか構築できないため)

# スタック削除時の注意
- 依存関係があるため、[構築順](#構築順) とは逆の順序で削除する
- data-store.yamlを削除する際は、RDSの削除保護がないか確認し、削除保護を無効化後に削除する

# スタック削除後の注意
スタックを削除した後も下記のリソースが残り、利用料金が掛かるので必要に応じて忘れないように削除する。
- RDSのスナップショット（RDSを削除する際に自動でスナップショットが作られる）
- S3バケット（スタック削除時に一緒に削除されないように設定しているバケットがある）
- CloudWatch Logs（スタック削除時に一緒に削除されないログがある）

# DBのマイグレーション方法
TODO

# APサーバの更新方法
TODO

# インフラリソースのスペックの更新方法
ymal内の該当するリソース（インスタンスタイプ等）を書き換えてCloudFormationにて更新する。更新の際は既存のリソースを削除する動作をしていないか確認するため、必ず変更セットを見て問題ないことがわかってから更新を行う。

# NOTE
Route53（Hosted Zone）、Systems Manager （※）、SESはCloudFormationの管理対象外としている（マネジメントコンソールから手動で構築している）

（※）下記の二種類を手動で対応する。
<ol>
  <li>パラメータストアにECSで利用するシークレットを含む環境変数をSecureStringのパラメータとして構築。SecureStringのパラメータは、テンプレート作成時点でCloudFormationに対応していない（作成は完全に未対応、読み込みも制限がある）ため手動で構築する。前述の通り使い勝手が悪いため、基本的に使わない想定でテンプレートを構築していた。しかし、ECSがコンテナを起動するとき、環境変数に平文としてシークレットを埋め込むのはリスクが高いと判断し、その箇所だけシークレットの読み込みをする際にのみ利用する。</li>
  <li>パラメータストアに収納代行用の銀行口座に関連する情報をSecureStringののパラメータとして構築。銀行口座に関連する情報は日本語を含んでおり、それらをCloudFormationテンプレート内で扱うことは困難である（CloudFormationテンプレートはASCII文字以外に対応していない）従って、それらの情報はパラメータストアに保管し、CloudFormationテンプレートからはパラメータストアを介して銀行口座に関連する情報を取得するようにする。StringでなくSecureStringを使う理由は、Cloudformationテンプレート内からパラメータストアのStringの値を参照する方法がないから。</li>
</ol>

## Route53（Hosted Zone）
下記の対応を行う
- CAレコードを作成する
- DNSSECに対応させる（DNSSECに対応させる際、KSKの管理にKMS上にCMKを作ることになる）
- DNSSECを有効にした際に監視すべき指標をCloudWatch Alarmに設定しておく

## Systems Manager
パラメータストアにSecureStringで下記のECSで利用するシークレットを含む環境変数のパラメータを作成する。
<ol>
  <li>prod-db-master-username (開発環境の場合は、dev-db-master-username)</li>
  <li>prod-db-master-password (開発環境の場合は、dev-db-master-password)</li>
  <li>prod-db-user-app-password (開発環境の場合は、dev-db-user-app-password)</li>
  <li>prod-db-admin-app-password (開発環境の場合は、dev-db-admin-app-password)</li>
  <li>prod-index-master-user (開発環境の場合は、dev-index-master-user)</li>
  <li>prod-index-master-password (開発環境の場合は、dev-index-master-password)</li>
  <li>prod-key-of-signed-cookie-for-user-app (開発環境の場合は、dev-key-of-signed-cookie-for-user-app)</li>
  <li>prod-key-of-signed-cookie-for-admin-app (開発環境の場合は、dev-key-of-signed-cookie-for-admin-app)</li>
  <li>prod-sky-way-application-id (開発環境の場合は、dev-sky-way-application-id)</li>
  <li>prod-sky-way-secret-key (開発環境の場合は、dev-sky-way-secret-key)</li>
</ol>

パラメータストアにSecureStringで下記の収納代行用の銀行口座に関連する情報のパラメータを作成する。
<ol>
  <li>prod-bank-code (開発環境の場合は、dev-bank-code)</li>
  <li>prod-bank-name (開発環境の場合は、dev-bank-name)</li>
  <li>prod-bank-branch-code (開発環境の場合は、dev-bank-branch-code)</li>
  <li>prod-bank-branch-name (開発環境の場合は、dev-bank-branch-name)</li>
  <li>prod-bank-account-number (開発環境の場合は、dev-bank-account-number)</li>
  <li>prod-bank-account-holder-name (開発環境の場合は、dev-bank-account-holder-name)</li>
</ol>

## SES
下記の対応を行う
- 問い合わせを受け付けるためのメールアドレスを用意し、アカウントを作成する。そして、メールの受信ができるようにRoute53にレコードを作成する（メール送信だけでなく、メール受信もできるようにus-east-1に構築する）
- DMARCに対応させる（SPF、DKIMの対応を行う）
- バウンスレートの指標をCloudWatch Alarmに設定しておく
