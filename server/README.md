# ローカルの開発環境のセットアップ

## 連携するサーバ群を立ち上げる
VS Code remote developmentで開発している場合、自動的に連携するサーバ群が立ち上がるため、特に対応は必要ない（連携するサーバ群の情報を含んだcompose.ymlが、プロジェクトルートの.devcontainer/devcontainer.jsonに記載されているため、VS Code remote development利用時に自動的に立ち上がる）<br>
<br>
VS Code remote developmentを使っていない場合、下記のコマンドで連携するサーバ群を立ち上げる。その後、.devcontainer/postCreateCommand.shに記載されているバックエンドの環境の準備処理を参考に各サーバの初期化を実施しておく。
```
docker compose up -d
```
連携するサーバを削除したくなった場合、下記のコマンドで削除する
```
docker compose down
```

## OSの設定値の変更
OpenSearchを安定して動作させるため、下記のリンクの設定に従い、（コンテナではなく）ホストマシン上のvm.max_map_countを262144以上に設定する。<br>
https://opensearch.org/docs/latest/opensearch/install/important-settings/

## 環境変数の用意
sample.envファイルを.envへリネームし、環境にあった変数を設定する

## 管理者アカウントのセットアップ
下記のコマンドを打ち、管理者向けサービスのアカウントを作成する
```
cargo run --bin admin_account create "管理者用Eメールアドレス" "パスワード"
```

## サービスの起動
下記のコマンドを打ち、ユーザ向けサービスを起動する
```
cargo run --bin user_service
```
下記のコマンドを打ち、管理者向けサービスを起動する
```
cargo run --bin admin_service
```

# ローカルの開発環境の更新
## DBのテーブルの変更と反映
開発中、DBのテーブル定義を更新したい場合、下記の項目を実施する。

### DBのテーブルを変更（DBのテーブル定義が記載されているソースコードを変更した場合、それをDBに反映する処理）
migration/src以下のソースコードを変更する。その後、下記のコマンドを実行する
```
export DATABASE_URL=postgres://postgres:example@db/ccs_db
sea-orm-cli migrate refresh
```

### 変更されたテーブルをentity以下のソースコードに反映（DBのテーブルをEntityを表すソースコードに反映する処理）
```
export DATABASE_URL=postgres://postgres:example@db/ccs_db
sea-orm-cli generate entity -l -s ccs_schema -o entity/src
```
Relationに関しては外部キーを使わないと自動で生成されない。従って、外部キーを使わずに関連を示したい場合、手動で記載する（sea-ormにおいて複数テーブルをJOINしたり、結果をINでまとめて再度SELECTして検索したい場合、関連が必要になる）

# 設計
## DBのテーブル設計方針

### トランザクション分離レベル
PostgreSQLのデフォルト（READ COMMITTED）を想定して設計。SERIALIZABLEではないので、トランザクション中でもアプリケーションで明示的なロック取得が必要なことを念頭に置く

### 外部キー採用時の検討事項
外部キーを採用するときは、下記の事項を検討し、課題がないことを明確にした上で採用する
#### パーティション化できなくなる
多くのDBでは、外部キーで関連付けたテーブルはパーティション化できなくなる。本プロジェクトで利用しているPostgreSQLも同様
#### 暗黙的ロックの発生を許容できる（アプリケーションコードに明示的にロックを取得するコードが出てこないことが許容できる）
子テーブルにinsertやupdateをかけたとき、親テーブルの対応するレコードに対し、暗黙的に共有ロックがかかる（アプリケーションコード上には子テーブルに対するinsertやupdateしか出てこない）
#### 外部キーのカラムに対してインデックス作成が許容できる
親テーブルのレコードの更新、削除で自動的に子テーブルのレコードの更新、削除が発生した場合、子テーブルの外部キーにインデックスがないと親テーブルの１レコードの操作に対して想定以上に遅い操作が発生する可能性がある。そのため、インデックスの作成が許容できる（インデックス再作成が少ない＝頻繁に値の更新がない）カラムであることを確認する
#### 親テーブルのレコードの更新に対して、子テーブルの更新が許容可能な時間内に操作が完了することを保証できる
ON DELETE CASCADE、ON DELETE SET NULL、ON UPDATE CASCADE、ON UPDATE SET NULLは、親テーブルの対応するレコードが更新されると自動的に子テーブルの更新がかかる（アプリケーションでは制御できない）そのため、親テーブルのレコードに紐づく子テーブルのレコードが無数に存在する場合、子テーブルの更新処理に多大な時間が費やされる可能性がある
#### 親子テーブル間で、外部キーで常に整合性を保つ必要があるほど強力な制約が必要か再考する
親テーブルのレコードが削除された場合を考える。子テーブルのレコードが、親テーブルを通してのみしかアクセスされない設計の場合、親テーブルのレコードが削除された時点で該当の子テーブルのレコードには既にアクセスできない。そのため、必ずしも同時に子テーブルのレコードを削除する必要はない。ここで説明するケースにおいては、バッチ処理で非同期的に子テーブルのレコードを削除すれば十分と考えられる

### ユーザー情報を扱う際の注意事項
ユーザーが利用する情報（ユーザー情報、職務経歴、相談料等々（※））は、user_accountテーブルを親として扱い、関連付けたテーブルを作成する。関連付ける際は、外部キーは使わない（参考：[外部キー採用時の検討事項](#外部キー採用時の検討事項)）user_accountテーブルからユーザーが削除された際、関連づいていた子テーブルのレコードは、非同期的にバッチ処理で削除する（削除の際はuser_accountテーブルからdeleted_user_accountテーブルへレコードを移動する。バッチ処理はdeleted_user_accountテーブルへレコードを読み取り、不要に成った子テーブルのレコード（※）を削除する）これの処理を考慮し、子テーブル -> user_accountテーブルの順序のデータの読み書きの流れを設計しないようにする。

（※）関連するテーブルの中でもユーザーが利用する情報のみ削除し、管理者が記録のために必要とするテーブルのデータは削除対象外とする（例えば、利用規約に同意したことを記録するテーブルは削除しない）

### トランザクション内におけるロックの取得順序
デッドロックとなる設計を避けるため、トランザクション内で複数ロックを取得する場合、取得するロックについてこのセクションに明記する。
デッドロックにならないように、下記の点を避けるように留意して設計する。
<ul>
  <li>あるトランザクションと別のトランザクションを比較したとき、たすき掛けになる順でロックを取得すること（※1）</li>
  <li>同一トランザクション内において、同一レコードに対して共有ロック、排他ロックの順でロックを取得すること（※2）</li>
</ul>
<p>（※1）取得するロックの種類（排他ロックと共有ロックの組み合わせ）を適切にすれば、たすき掛けでもデッドロックは起こらないが、問題が見つかりにくくなるため、たすき掛けになるロックの取得順は一律で禁止とする</p>
<p>（※2）外部キーを使う場合、子のレコードをINSERT、UPDATEするときに親レコードに暗黙的に共有ロックがかかる。このケースは見落としやすいので要注意する</p>

#### admin_service
##### handlers/session/authentication/authenticated_handlers/identity_request/create_request/approval.rs
user_accountで共有ロックを取得 -> create_identity_reqで排他ロックを取得
##### handlers/session/authentication/authenticated_handlers/identity_request/create_request/rejection.rs
user_accountで共有ロックを取得 -> create_identity_reqで排他ロックを取得
##### handlers/session/authentication/authenticated_handlers/identity_request/update_request/approval.rs
user_accountで共有ロックを取得 -> identityで排他ロックを取得 -> update_identity_reqで排他ロックを取得
##### handlers/session/authentication/authenticated_handlers/identity_request/update_request/rejection.rs
user_accountで共有ロックを取得 -> update_identity_reqで排他ロックを取得
##### handlers/session/authentication/authenticated_handlers/career_request/create_request/approval.rs
user_accountで共有ロックを取得 -> create_career_reqで排他ロックを取得 -> documentで共有ロックを取得
##### handlers/session/authentication/authenticated_handlers/career_request/create_request/rejection.rs
user_accountで共有ロックを取得 -> create_career_reqで排他ロックを取得
##### handlers/session/authentication/authenticated_handlers/user_account/disable_user_account_req.rs
user_accountで排他ロックを取得 -> documentで排他ロックを取得

#### user_service
##### handlers/session/authentication/authenticated_handlers/personal_info/profile/fee_per_hour_in_yen.rs
consulting_feeで排他ロックを取得 -> documentで共有ロックを取得
##### handlers/session/authentication/authenticated_handlers/consultation/rating/consultant_rating.rs
user_accountで排他ロックを取得 -> documentで共有ロックを取得
##### handlers/session/authentication/authenticated_handlers/delete_accounts.rs
user_accountで排他ロックを取得 -> documentで排他ロックを取得

## 検索用インデックスの設計について
検索用インデックスには、OpenSearchを利用する。検索用インデックスに投入するデータは、DBの値、もしくはその値を加工して生成できる値に限定する（検索用インデックスを一次データの保管場所として採用しない）
