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
12. deploy-user.yamlを使いスタックを構築する
13. 前項のスタックができると[リリース](#リリース)で利用するIAMユーザーができるので、そのユーザーのアクセスキーIDとシークレットアクセスキーをWeb UIから発行する
14. [DB初期化](#DB初期化)の項目を実施する
15. [マイグレーション](#マイグレーション)の項目を実施する
16. [インデックス初期化](#インデックス初期化)の項目を実施する
17. [管理者用アカウントの作成](#管理者用アカウントの作成)の項目を実施する
18. admin-service.yamlで作成したスタックのInstanceCountを1、user-service.yamlで作成したスタックのMinInstanceCountを1、MaxInstanceCountを8にし、delete-expired-xxx.yamlで作成したスタックのScheduledTaskEnabledはtrueに更新する
19. [リリースビルド](#リリースビルド)を実施した結果、フロントエンドのリリース用コードがCIの結果格納用のS3バケットにアップロードされているので、それをリリース用のバケットにコピーする（ユーザー向けはccs-user-app-ci-result-storageからxxx-ccs-user-appへコピーし、管理者向けはccs-admin-app-ci-result-storageからxxx-ccs-admin-appへコピーする）

## 環境構築時の注意
### TLS証明書初回発行の際のドメイン検証
TLS証明書発行の際、証明書の発行者がその証明書に記載するドメインを保持しているかどうか検証が行われる。従って、TLS証明書の発行を伴うスタック構築は、その検証に対応する必要がある（検証方法には種類があるが、本環境のテンプレートは全てドメイン検証で検証が行われるように作られているため、ドメイン検証の対応を行う）検証には下記の手順に従い対応する。なお、検証は初めて証明書を発行しようとした際に発生する。そのため、検証が完了しスタックが構築された後、そのスタックを削除し、その後同じスタックを構築しようとした場合は発生しない。
1. 証明書発行を伴うスタック構築中にUPDATE_IN_PROGRESSの状態でドメイン検証待ちの状態となっていること確認する（スタックのイベントからドメイン検証待ちのイベントがあることを確認可能）
2. Web UIからドメイン検証を行う
3. UPDATE_IN_PROGRESSの状態でドメイン検証待ちの状態となっていたスタックが構築完了することを確認する

### スタックを構築するリージョン
#### request-controller.yaml以外のCloudFormationテンプレート
- ap-northeast-1

#### request-controller.yaml
- us-east-1 (CloudFrontに紐づくWebACLやACMがus-east-1でしか構築できないため)

# リリースビルド
1. リリースビルド対象のコミットに対して、gitでtagをつけておく（タグは、release-n (nは1以上の数字) という形式でnをインクリメントする形で新しいタグを作っていく）
2. 前項のtagを指定して、Github Actionsの"Test, build and upload artifacts"を実行する（これはソースコードの静的解析、単体テスト、リリースビルドを行い、その成果物をAWS上のリリース準備用の場所にアップロードする。無料利用枠がなくなりGithub Actionsを使えない場合、[該当コード](../.github/workflows/ci.yaml)を参照し、ローカルで同じ処理を行う）

# リリース
1. [リリースビルド](#リリースビルド)が完了していることを確認する
2. 切り戻しが発生した場合に備えて、下記のコマンドでフロントエンドのコードをローカルにバックアップしておく（アクセスキーIDとシークレットはdeploy-user.yamlで作られるユーザーのものを使う）
   <ul>
     <li>ユーザー向けフロントエンドコード: aws s3 sync s3://prod-ccs-user-app "ローカルのディレクトリ" --exact-timestamps</li>
     <li>管理者向けフロントエンドコード: aws s3 sync s3://prod-ccs-admin-app "ローカルのディレクトリ" --exact-timestamps</li>
   </ul>
5. [リリースビルド](#リリースビルド)で指定したtagで、Github Actionsの"Update setup tool"を実行する（これはマイグレーション処理（＋その他の処理）が記載されているタスク定義を更新する。正しく完了したかどうかはGithub ActionsとCloudFormationのスタックを確認する。無料利用枠がなくなりGithub Actionsを使えない場合、[該当コード](../.github/workflows/cd-setup-tool.yaml)を参照し、ローカルで同じ処理を行う）
6. サービス停止が必要な場合、[サービスの停止](#サービスの停止)を実行する
7. マイグレーションが必要な場合、[マイグレーション](#マイグレーション)の項目を実施する
8. [リリースビルド](#リリースビルド)で指定したtagで、Github Actionsの"Update application"を実行する（正しく完了したかどうかはGithub Actions、CloudFormationのスタック、Webページの実際の表示で確認する。無料利用枠がなくなりGithub Actionsを使えない場合、[該当コード](../.github/workflows/cd-application.yaml)を参照し、ローカルで同じ処理を行う）
9. サービス停止をしている場合、admin-service.yamlで作成したスタックのInstanceCountを1、user-service.yamlで作成したスタックのMinInstanceCountを1、MaxInstanceCountを8にし、delete-expired-xxx.yamlで作成したスタックのScheduledTaskEnabledはtrueに更新する（フロントエンドのコードは自動的にデプロイされているため、バックエンドの対応のみ行う）

# 切り戻し
[リリース](#リリース)が完了し、動作確認した結果NGだった場合の手続きを記載する。
1. サービス停止が必要な場合、[サービスの停止](#サービスの停止)を実行する
2. マイグレーションをしていた場合、[ロールバック](#ロールバック)の項目を実施する
3. [リリース](#リリース)で指定したタグから一つ前のバージョンのタグを指定し、Github Actionsの"Update setup tool"を実行する（正しく完了したかどうかはGithub ActionsとCloudFormationのスタックを確認する。無料利用枠がなくなりGithub Actionsを使えない場合、[該当コード](../.github/workflows/cd-setup-tool.yaml)を参照し、ローカルで同じ処理を行う）
4. [リリース](#リリース)で指定したタグから一つ前のバージョンのタグを指定し、Github Actionsの"Update application"を実行する（正しく完了したかどうかはGithub Actions、CloudFormationのスタック、Webページの実際の表示で確認する。無料利用枠がなくなりGithub Actionsを使えない場合、[該当コード](../.github/workflows/cd-application.yaml)を参照し、ローカルで同じ処理を行う）
5. 切り戻しに備えてローカルにバックアップしていたフロントエンドのコードを下記のコマンドでアップロードする（アクセスキーIDとシークレットはdeploy-user.yamlで作られるユーザーのものを使う）
   <ul>
     <li>ユーザー向けフロントエンドコード: aws s3 sync "ローカルのディレクトリ" s3://prod-ccs-user-app --delete</li>
     <li>管理者向けフロントエンドコード: aws s3 sync "ローカルのディレクトリ" s3://prod-ccs-admin-app --delete</li>
   </ul>
6. サービス停止をしている場合、admin-service.yamlで作成したスタックのInstanceCountを1、user-service.yamlで作成したスタックのMinInstanceCountを1、MaxInstanceCountを8にし、delete-expired-xxx.yamlで作成したスタックのScheduledTaskEnabledはtrueに更新する（前項の対応でフロントエンドのコードはデプロイ済みのため、バックエンドの対応のみ行う）
7. 必要に応じてユーザー向けフロントエンドコードと管理者向けフロントエンドコードを提供しているCloudFrontのキャッシュ無効化を行う

# DB初期化
ファミリーの項目にdb-initializer-task.yamlで作成されたタスク定義、リビジョンに最新バージョンを指定し、[手動でのタスクの実行](#手動でのタスクの実行)を実施する。

# マイグレーション
ファミリーの項目にmigration-tool-migrate-task.yamlで作成されたタスク定義、リビジョンに最新バージョンを指定し、[手動でのタスクの実行](#手動でのタスクの実行)を実施する。

# ロールバック
ファミリーの項目にmigration-tool-rollback-task.yamlで作成されたタスク定義、リビジョンに最新バージョンを指定し、[手動でのタスクの実行](#手動でのタスクの実行)を実施する。

# インデックス初期化
ファミリーの項目にindex-initializer-task.yamlで作成されたタスク定義、リビジョンに最新バージョンを指定し、[手動でのタスクの実行](#手動でのタスクの実行)を実施する。

# 管理者用アカウントの作成
必ずアカウント作成時に二段階認証の有効化まで行う。
## アカウント作成
下記の設定を使い、 [手動でのタスクの実行](#手動でのタスクの実行)を実行する
1. ファミリーの項目にadmin-account-task.yamlで作成されたタスク定義、リビジョンに最新バージョンを指定する
2. 「コンテナの上書き」の「コマンドの上書き」にサブコマンド (create) と引数 (メールアドレス、パスワード) をカンマ区切りリストで記載する（アカウント作成の際の例: create,admin@example.com,vvvvvvvvvV）

## 二段階認証有効化
下記の設定を使い、 [手動でのタスクの実行](#手動でのタスクの実行)を実行する。実行後、CloudWatch Logs（/ecs/xxx-ccs-admin-account (xxxはprodまたはdev)）にhtmlファイルの形式でシークレットが出力される。その出力をCSVとしてダウンロードし、必要な2行目、2列目以降をコピーし、ローカルにsecret.htmlとして保存する。そのファイルをブラウザで開くとQRコードが表示されるので、それをスマートフォンの二段階認証用アプリで読み取る
1. ファミリーの項目にadmin-account-task.yamlで作成されたタスク定義、リビジョンに最新バージョンを指定する
2. 「コンテナの上書き」の「コマンドの上書き」にサブコマンド (mfa) と引数 (enable、メールアドレス) をカンマ区切りリストで記載する（二段階認証有効化の際の例: mfa,enable,admin@example.com）

## 手動でのタスクの実行
1. マネジメントコンソールから「Amazon Elastic Container Service > クラスター > xxx-ccs-application-cluster (xxxはprodまたはdev) > タスク」へ移動する
2. 「新しいタスクの実行」をクリックする
3. 「ファミリー」に実行したいタスク定義を入れ、「リビジョン」は最新を指定する
4. ネットワーキング内のVPCは「xxx-ccs-vpc」、サブネットは「xxx-ccs-public-subnet1-ap-north-east-1a」、セキュリティグループは「XxxCcsToolSecutiryGroup」を指定する (xxxはprodまたはdev、XxxはProdまたはDev)
5. 「タスクの実行」をクリックする
6. クラスター内のタスクとCloudWatch Logsで実行結果を確認する

# サービスの停止
1. admin-service.yamlで作成したスタックのInstanceCountを0、user-service.yamlで作成したスタックのMinInstanceCountとMaxInstanceCountを0にし、delete-expired-xxx.yamlで作成したスタックのScheduledTaskEnabledはfalseに更新する
2. ユーザー向けフロントエンドコードを保管しているバケットを空にする
3. [メンテナンス用のページ](maintenance_page/index.html)をユーザー向けフロントエンドコードを保管しているバケットにアップロードする
4. 必要に応じてユーザー向けフロントエンドコードを提供しているCloudFrontのキャッシュ無効化を行う

# 環境の削除
1. RDSにあるデータベースが削除保護の設定がないか確認し、削除保護されている場合はそれを解除する
2. S3のxxx-ccs-user-appバケットとxxx-ccs-admin-appバケット内にファイルがある場合、空にしておく (xxxはprodまたはdev)
3. [環境の構築](#環境の構築)で構築したのとは逆順にスタックを削除していく
4. 全てのスタックが削除された後、RDSのスナップショット、スタック削除では一緒に削除されないS3バケット、不要なCloudWatch Logsを削除する

# インフラリソースのスペックの更新方法
ymal内の該当するリソース（インスタンスタイプ等）を書き換えてCloudFormationにて更新する。更新の際は既存のリソースを削除する動作をしていないか確認するため、必ず変更セットを見て問題ないことがわかってから更新を行う。

# CloudFormationの管理対象外のリソース
Route53（Hosted Zone）、SES、Systems ManagerはCloudFormationの管理対象外としている（マネジメントコンソールから手動で構築している）下記に対象サービスに対して手動で実施する必要のある内容を記載する。

## Route53（Hosted Zone）
下記の対応を行う
- CAレコードを作成する
- DNSSECに対応させる（DNSSECに対応させる際、KSKの管理にKMS上にCMKを作ることになる）
- DNSSECを有効にした際に監視すべき指標をCloudWatch Alarmに設定しておく

## SES
下記の対応を行う
- 問い合わせを受け付けるためのメールアドレスを用意し、アカウントを作成する。そして、メールの受信ができるようにRoute53にレコードを作成する（メール送信だけでなく、メール受信もできるようにus-east-1に構築する）
- DMARCに対応させる（SPF、DKIMの対応を行う）
- バウンスレートの指標をCloudWatch Alarmに設定しておく

## Systems Manager
パラメータストアにSecureStringで下記のパラメータを作成する（SecureStringのパラメータは、テンプレート作成時点でCloudFormationに対応していない（作成は完全に未対応、読み込みも制限がある）ため手動で構築する。xxx-bank-xxxの名称のパラメータは、収納代行用の銀行口座に関連する情報のパラメータ）
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
  <li>prod-bank-code (開発環境の場合は、dev-bank-code)</li>
  <li>prod-bank-name (開発環境の場合は、dev-bank-name)</li>
  <li>prod-bank-branch-code (開発環境の場合は、dev-bank-branch-code)</li>
  <li>prod-bank-branch-name (開発環境の場合は、dev-bank-branch-name)</li>
  <li>prod-bank-account-number (開発環境の場合は、dev-bank-account-number)</li>
  <li>prod-bank-account-holder-name (開発環境の場合は、dev-bank-account-holder-name)</li>
</ol>
