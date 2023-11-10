# 環境の構築
下記の手順に従い、環境を構築する。環境を構築する際、利用する各テンプレートのEnvironmentパラメータは手順を通して統一する（本番環境であればprod、開発環境であればdev）テンプレート毎にスタックを構築するリージョンが異なるため注意する（詳細はを参照）
1. [CloudFormationの管理対象外のリソース](#CloudFormationの管理対象外のリソース)に記載のリソースを手動で構築する
2. artifacts-store.yamlを使いスタックを構築する（prodとdevで共通のスタックを使うため、既に作成済の場合は、この項と次項はスキップする）
3. 前項のスタックができると[リリースビルド](#リリースビルド)で利用するIAMユーザーができるので、そのユーザーのアクセスキーIDとシークレットアクセスキーをWeb UIから発行する
4. [リリースビルド](#リリースビルド)の項目を実施する
5. network.yamlを使いスタックを構築する
6. static-files.yamlを使いスタックを構築する
7. data-store.yamlを使いスタックを構築する（postgresのユーザー名とパスワード、OpenSearchのユーザー名とパスワードは[Systems Manager](#Systems-Manager)で構築した際の値と同じものにする）
8. load-balancer.yamlを使いスタックを構築する（2種類のカスタムヘッダの値には任意の値を入力する。ただし、それぞれ異なる値、かつ予測が困難なものとすること。初めてスタックを構築する際は[TLS証明書初回発行の際のドメイン検証](#TLS証明書初回発行の際のドメイン検証)を参照すること）
9. application-cluster.yamlを使いスタックを構築する
10. applicationsディレクトリ以下の各テンプレートを使い、スタックを構築する（ImageTagパラメータには[リリースビルド](#リリースビルド)にてtagをつけたコミットのコミットIDを指定する。またPostgresとOpenSearchの初期化が終わっていないため、それらにアクセスしないようパラメータを指定する（admin-service.yamlのInstanceCount、user-service.yamlのMinInstanceCountとMaxInstanceCountは0に、delete-expired-xxx.yamlのScheduledTaskEnabledはfalseを指定する））
11. request-controller.yamlを使いスタックを構築する（**us-east-1**で構築する。2種類のカスタムヘッダの値にはload-balancer.yamlでスタックを構築した際に利用したものを同じ値を使う）

## 環境構築時の注意
### TLS証明書初回発行の際のドメイン検証
TLS証明書発行の際、証明書の発行者がその証明書に記載するドメインを保持しているかどうか検証が行われる。従って、TLS証明書の発行を伴うスタック構築は、その検証に対応する必要がある（検証方法には種類があるが、本環境のテンプレートは全てドメイン検証で検証が行われるように作られているため、ドメイン検証の対応を行う）検証には下記の手順に従い対応する。なお、検証は初めて証明書を発行しようとした際に発生する。そのため、検証が完了しスタックが構築された後、そのスタックを削除し、その後同じスタックを構築しようとした場合は発生しない。
1. 証明書発行を伴うスタック構築中にUPDATE_IN_PROGRESSの状態でドメイン検証待ちの状態となっていること確認する（スタックのイベントからドメイン検証待ちのイベントがあることを確認可能）
2. Web UIからドメイン検証を行う
3. UPDATE_IN_PROGRESSの状態でドメイン検証待ちの状態となっていたスタックが構築完了することを確認する

### スタックを構築するリージョン
スタックを構築する際、テンプレートに対して構築するリージョンが決まっているため注意する
## ap-northeast-1
- request-controller.yaml以外のCloudFormationテンプレート

## us-east-1
- request-controller.yaml (CloudFrontに紐づくWebACLやACMがus-east-1でしか構築できないため)

# リリースビルド
1. リリースビルド対象のコミットに対して、gitでtagをつけておく
2. ソースコードの静的解析、単体テスト、リリースビルドを行い、その成果物をAWS上にアップロードする（）

# 構築順
依存関係があるため構築順序の通りに構築する必要がある。
1. [CloudFormationの管理対象外のリソース](#CloudFormationの管理対象外のリソース) に記載のリソースを手動で構築
2. artifacts-store.yaml
3. network.yaml
4. static-files.yaml
5. data-store.yaml
6. load-balancer.yaml
7. application-cluster.yaml
8. applicationsディレクトリ以下の全てのCloudFormationテンプレート
9. request-controller.yaml
10. deploy-user.yaml

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

# リリースビルド、リリース用Dockerイメージの作成
Github Actionsにてリリースビルド、リリース用Dockerイメージの作成を行っているため、[該当コード](../.github/workflows/ci.yaml)を参照

# インフラリソースのスペックの更新方法
ymal内の該当するリソース（インスタンスタイプ等）を書き換えてCloudFormationにて更新する。更新の際は既存のリソースを削除する動作をしていないか確認するため、必ず変更セットを見て問題ないことがわかってから更新を行う。

# CloudFormationの管理対象外のリソース
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
