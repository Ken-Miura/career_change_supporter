// Copyright 2021 Ken Miura

use entity::sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let db_backend = manager.get_database_backend();
        let sql = Sql::new(db_backend);

        // SCHEMA定義
        let _ = conn
            .execute(sql.stmt(r"CREATE SCHEMA ccs_schema;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT USAGE ON SCHEMA ccs_schema TO user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT USAGE ON SCHEMA ccs_schema TO admin_app;"))
            .await
            .map(|_| ())?;
        // DOMAIN定義
        let _ = conn
            .execute(sql.stmt(r"CREATE DOMAIN ccs_schema.email_address AS VARCHAR (254) CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );"))
            .await
            .map(|_| ())?;
        let _ = conn
            /* simpleフォーム (半角英数字32文字。ハイフン、波括弧を含まない) での入出力を行いたいので、標準のUUID型を使わない */
            .execute(sql.stmt(r"CREATE DOMAIN ccs_schema.uuid_simple_form AS CHAR (32) CHECK ( VALUE ~ '^[a-zA-Z0-9]+$' );"))
            .await
            .map(|_| ())?;
        let _ = conn
            /* 
             * regular: 正社員、contract: 契約社員、other: その他
             */
            .execute(sql.stmt(r"CREATE DOMAIN ccs_schema.contract_type AS VARCHAR (8) CHECK (VALUE ~ 'regular' OR VALUE ~ 'contract' OR VALUE ~ 'other');"))
            .await
            .map(|_| ())?;
        // その他（TABLE、INDEX等）の定義
        let _ = conn
        /* ユーザーがアカウントを作成した際に生成される。ユーザーがアカウントを削除した際に削除される（情報は削除されたユーザーテーブルに移される） */
        /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
        .execute(sql.stmt(r"CREATE TABLE ccs_schema.user_account (
            user_account_id BIGSERIAL PRIMARY KEY,
            email_address ccs_schema.email_address NOT NULL UNIQUE,
            hashed_password BYTEA NOT NULL,
            last_login_time TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            mfa_enabled_at TIMESTAMP WITH TIME ZONE,
            disabled_at TIMESTAMP WITH TIME ZONE
          );"))
        .await
        .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT, DELETE ON ccs_schema.user_account To user_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT UPDATE (hashed_password, last_login_time, mfa_enabled_at) ON ccs_schema.user_account To user_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT ON ccs_schema.user_account To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT UPDATE (mfa_enabled_at, disabled_at) ON ccs_schema.user_account To admin_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            /*
             * NOTE: 下記の参考によると、SERIALで暗黙的に作成されるSEQUENCEはtablename_colname_seqで定められる
             * 参考: https://www.postgresql.org/docs/13/datatype-numeric.html#DATATYPE-SERIAL
             */
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.user_account_user_account_id_seq TO user_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーがアカウントを削除した際に生成される。一定期間後、定期実行ツールにより削除される */
            /* NOTE: アカウント作成 -> 削除 -> バッチ処理によりdeleted_user_accountが削除される前に同じメールアドレスでアカウント作成 -> 削除
             * といったケースに対応するためにemail_addressにはUNIQUEをつけない
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.deleted_user_account (
                user_account_id BIGINT PRIMARY KEY,
                email_address ccs_schema.email_address NOT NULL,
                hashed_password BYTEA NOT NULL,
                last_login_time TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                mfa_enabled_at TIMESTAMP WITH TIME ZONE,
                disabled_at TIMESTAMP WITH TIME ZONE,
                deleted_at TIMESTAMP WITH TIME ZONE NOT NULL
              );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE ON ccs_schema.deleted_user_account To user_app;",
            ))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.deleted_user_account To admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーがアカウントの作成を試みたときに生成される。有効期限が切れた後、定期実行ツールにより削除される */
            /* 一度仮登録した後、それを忘れてしまいもう一度仮登録したいケースを考え、email_addressをUNIQUEにしない。user_temp_account_idがPRIMARY KEYなので一意に検索は可能 */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.user_temp_account (
                    user_temp_account_id ccs_schema.uuid_simple_form PRIMARY KEY,
                    email_address ccs_schema.email_address NOT NULL,
                    hashed_password BYTEA NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL
                  );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.user_temp_account To user_app;"))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.user_temp_account To admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが利用規約に同意したときに生成される。サービスの運用期間を通じて存在し続ける */
            /*
             * ユーザーが利用規約に同意した証拠となる。
             * そのため、後から同意したことを追跡できるように、アカウントが削除されても利用規約の合意は削除されないようにする
             *（user_account_idを外部キーとしてuser_account.user_account_idと関連付けない）
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.terms_of_use (
             user_account_id BIGINT NOT NULL,
             ver INTEGER NOT NULL,
             email_address ccs_schema.email_address NOT NULL,
             agreed_at TIMESTAMP WITH TIME ZONE NOT NULL,
             PRIMARY KEY (user_account_id, ver)
           );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.terms_of_use To user_app;"))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.terms_of_use To admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーがパスワードの変更を試みたときに生成される。有効期限が切れた後、定期実行ツールにより削除される */
            /* 一度パスワード変更依頼を出した後、もう一度パスワード変更依頼を出したいケースを考慮し、email_addressをUNIQUEにしない。pwd_change_req_idがPRIMARY KEYなので一意に検索は可能 */
            .execute(sql.stmt(r"CREATE TABLE ccs_schema.pwd_change_req (
                pwd_change_req_id ccs_schema.uuid_simple_form PRIMARY KEY,
                email_address ccs_schema.email_address NOT NULL,
                requested_at TIMESTAMP WITH TIME ZONE NOT NULL
              );"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.pwd_change_req To user_app;"))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.pwd_change_req To admin_app;"))
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが二段階認証を有効にしようとしたときに生成される。ユーザーが二段階認証を有効にしたときに削除される。
             * 有効にしようと試みた後、実際に有効にせずに残っていたものは、有効期限が切れた後、定期実行ツールにより削除される
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.temp_mfa_secret (
                temp_mfa_secret_id BIGSERIAL PRIMARY KEY,
                user_account_id BIGINT NOT NULL,
                base32_encoded_secret TEXT NOT NULL,
                expired_at TIMESTAMP WITH TIME ZONE NOT NULL
              );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"GRANT SELECT, INSERT, DELETE ON ccs_schema.temp_mfa_secret To user_app;",
                ),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.temp_mfa_secret To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.temp_mfa_secret_temp_mfa_secret_id_seq TO user_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX temp_mfa_secret_user_account_id_idx ON ccs_schema.temp_mfa_secret (user_account_id);",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX temp_mfa_secret_expired_at_idx ON ccs_schema.temp_mfa_secret (expired_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが二段階認証を有効にしたときに生成される。ユーザーが二段階認証を無効にしたとき、リカバリーコードでログインしたときに削除される */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.mfa_info (
                user_account_id BIGINT PRIMARY KEY,
                base32_encoded_secret TEXT NOT NULL,
                hashed_recovery_code BYTEA NOT NULL
              );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT, DELETE ON ccs_schema.mfa_info To user_app;"))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.mfa_info To admin_app;"))
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がユーザーの身分確認依頼を承認したときに生成される。
             * ユーザーがアカウントを削除した後（削除されたユーザーテーブルに移された後）一定期間後、定期実行ツールにより削除される
             */
            /* user_account一つに対して、identityは0もしくは1の関係とする。 */
            /* prefecture => 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
            /* city => 市区町村の最大文字数は6文字。しかし、市区町村は頻繁に名前が変更される可能性があるので長さに余裕をもたせる */
            /*
             * telephone_number
             * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
             * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
             */
            .execute(sql.stmt(r"CREATE TABLE ccs_schema.identity (
              user_account_id BIGINT PRIMARY KEY,
              last_name VARCHAR (64) NOT NULL,
              first_name VARCHAR (64) NOT NULL,
              last_name_furigana VARCHAR (64) NOT NULL,
              first_name_furigana VARCHAR (64) NOT NULL,
              date_of_birth DATE NOT NULL,
              prefecture VARCHAR (4) NOT NULL,
              city VARCHAR (32) NOT NULL,
              address_line1 VARCHAR (128) NOT NULL,
              address_line2 VARCHAR (128),
              telephone_number VARCHAR (13) NOT NULL
            );"))
            .await
            .map(|_| ())?;
        let _ = conn
            /* 身分情報は、管理者 (admin_app) が提出されたエビデンスを確認し、レコードを挿入、更新する。従って、ユーザー (user_app) には挿入、更新権限は持たせない。*/
            /* アカウント削除はユーザー自身が行う。しかし、紐付いた情報は定期実行ツールで削除するためDELETE権限は付与しない */
            .execute(sql.stmt(r"GRANT SELECT ON ccs_schema.identity To user_app;"))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ =
            conn.execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.identity To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            /*
             * 管理者が本人確認の際に、既に登録済のユーザー情報があるかどうか確認する必要がある。
             * 既に登録済のユーザー情報かどうかを調べるため、生年月日が一致するユーザー情報を抽出 (*) し、その他の項目が一致するか確認し、ユーザー情報が既に登録済か確認する。
             * (*) 生年月日がユーザー情報の中で最も一致率が低いと考えられるため、生年月日を利用する
             *（住所を示す複数カラムにインデックスを張る選択肢もあるが、住所は変更され、インデックス張り直しの可能性があるため避ける）
             */
            .execute(sql.stmt(
                r"CREATE INDEX identity_date_of_birth_idx ON ccs_schema.identity (date_of_birth);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            .execute(
                /* 管理者がユーザーの職歴確認依頼を承認したときに生成される。また、ユーザーが職歴を削除した際に削除される。
                 * ユーザーがアカウントを削除した後（削除されたユーザーテーブルに移された後）一定期間後、定期実行ツールにより削除される
                 */
                /* annual_income_in_man_yen => 万円単位での年収 */
                sql.stmt(
                    r"CREATE TABLE ccs_schema.career (
                    career_id BIGSERIAL PRIMARY KEY,
                    user_account_id BIGINT NOT NULL,
                    company_name VARCHAR (256) NOT NULL,
                    department_name VARCHAR (256),
                    office VARCHAR (256),
                    career_start_date DATE NOT NULL,
                    career_end_date DATE,
                    contract_type ccs_schema.contract_type NOT NULL,
                    profession VARCHAR (128),
                    annual_income_in_man_yen INTEGER,
                    is_manager BOOLEAN NOT NULL,
                    position_name VARCHAR (128),
                    is_new_graduate BOOLEAN NOT NULL,
                    note VARCHAR (2048)
                  );",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                /* 職務経歴は、管理者 (admin_app) が提出されたエビデンスを確認し、レコードを挿入、更新する。従って、ユーザー (user_app) には挿入、更新権限は持たせない。*/
                /* 職務経歴の削除はユーザー自身でも可能。そのため削除権限をユーザー (user_app) に付与する */
                sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.career To user_app;"),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(
                sql.stmt(
                    r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.career To admin_app;",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT USAGE ON SEQUENCE ccs_schema.career_career_id_seq TO admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが相談料を（初めて）設定したときに生成される。
             * ユーザーがアカウントを削除した後（削除されたユーザーテーブルに移された後）一定期間後、定期実行ツールにより削除される
             */
            /* user_account一つに対して、consulting_feeは0もしくは1の関係とする。 */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.consulting_fee (
                  user_account_id BIGINT PRIMARY KEY,
                  fee_per_hour_in_yen INTEGER NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT, UPDATE ON ccs_schema.consulting_fee To user_app;"),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(
                sql.stmt(
                    r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.consulting_fee To admin_app;",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが（初めて）入金口座の登録をしたときに生成される。
             * ユーザーがアカウントを削除した後（削除されたユーザーテーブルに移された後）一定期間後、定期実行ツールにより削除される
             */
            /*
             * user_account一つに対して、bank_accountは0もしくは1の関係とする。
             * 口座に関する情報は仕様が統一されていないように見える。従って口座情報に関してはTEXT型で保存し、把握していない仕様が後から判明しても対応できるようにする。
             * 現状把握している口座情報の制限はアプリケーションコードのバリデーションチェックで対応する。
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.bank_account (
                  user_account_id BIGINT PRIMARY KEY,
                  bank_code TEXT NOT NULL,
                  branch_code TEXT NOT NULL,
                  account_type TEXT NOT NULL,
                  account_number TEXT NOT NULL,
                  account_holder_name TEXT NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT, UPDATE ON ccs_schema.bank_account To user_app;"),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.bank_account To admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが相談申し込みをしたときに生成される。コンサルタントが相談申し込みを承認、または拒否したときに削除される。
             * 相談開始日時の候補すべてが現在時刻を超えている場合、定期実行ツールにより削除される
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.consultation_req (
                  consultation_req_id BIGSERIAL PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  first_candidate_date_time TIMESTAMP WITH TIME ZONE NOT NULL,
                  second_candidate_date_time TIMESTAMP WITH TIME ZONE NOT NULL,
                  third_candidate_date_time TIMESTAMP WITH TIME ZONE NOT NULL,
                  latest_candidate_date_time TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.consultation_req To user_app;",
            ))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ =
            conn.execute(sql.stmt(
                r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.consultation_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.consultation_req_consultation_req_id_seq TO user_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX consultation_req_latest_candidate_date_time_idx ON ccs_schema.consultation_req (latest_candidate_date_time);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* コンサルタントが相談申し込みを承認したときに生成される。サービスの運用期間を通じて存在し続ける。不要なデータは相談日時でフィルタリングして利用する */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.consultation (
                  consultation_id BIGSERIAL PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  room_name ccs_schema.uuid_simple_form NOT NULL UNIQUE,
                  user_account_entered_at TIMESTAMP WITH TIME ZONE,
                  consultant_entered_at TIMESTAMP WITH TIME ZONE,
                  UNIQUE(user_account_id, meeting_at),
                  UNIQUE(consultant_id, meeting_at)
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT, UPDATE ON ccs_schema.consultation To user_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT ON ccs_schema.consultation To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.consultation_consultation_id_seq TO user_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX consultation_user_account_id_idx ON ccs_schema.consultation (user_account_id);",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX consultation_consultant_id_idx ON ccs_schema.consultation (consultant_id);",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX consultation_meeting_at_idx ON ccs_schema.consultation (meeting_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * コンサルタントが相談申し込みを承認したときに生成される。
             * 管理者がユーザーからの支払いを確認したとき削除される。
             * 管理者がユーザーからの返金依頼を処理したときに削除される（返金を受け付けるのは、ユーザーが相談日時までに入金したにも関わらず、管理者が支払いの確認を出来なかった場合のみ）
             * 管理者が相談日時までにユーザーからの入金を確認できなかったときに削除される
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.awaiting_payment (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL,
                  created_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.awaiting_payment To user_app;"))
            .await
            .map(|_| ())?;
        let _ =
            conn.execute(sql.stmt(
                r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.awaiting_payment To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_payment_user_account_id_idx ON ccs_schema.awaiting_payment (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_payment_consultant_id_idx ON ccs_schema.awaiting_payment (consultant_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_payment_meeting_at_idx ON ccs_schema.awaiting_payment (meeting_at);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_payment_created_at_idx ON ccs_schema.awaiting_payment (created_at);",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 管理者がユーザーの入金を確認したときに生成される。サービスの運用期間を通じて存在し続ける。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             *  - このテーブルに検索に使えるカラムがないため、このテーブルを左にして結合する場合、処理が遅くなる懸念がある
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.user_rating (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  rating SMALLINT,
                  rated_at TIMESTAMP WITH TIME ZONE
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, UPDATE ON ccs_schema.user_rating To user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.user_rating To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX user_rating_user_account_id_idx ON ccs_schema.user_rating (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX user_rating_consultant_id_idx ON ccs_schema.user_rating (consultant_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX user_rating_meeting_at_idx ON ccs_schema.user_rating (meeting_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 管理者がユーザーの入金を確認したときに生成される。サービスの運用期間を通じて存在し続ける。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             *  - このテーブルに検索に使えるカラムがないため、このテーブルを左にして結合する場合、処理が遅くなる懸念がある
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.consultant_rating (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  rating SMALLINT,
                  rated_at TIMESTAMP WITH TIME ZONE
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, UPDATE ON ccs_schema.consultant_rating To user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.consultant_rating To admin_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX consultant_rating_user_account_id_idx ON ccs_schema.consultant_rating (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX consultant_rating_consultant_id_idx ON ccs_schema.consultant_rating (consultant_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX consultant_rating_meeting_at_idx ON ccs_schema.consultant_rating (meeting_at);",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 管理者がユーザーの入金を確認したときに生成される。
             * 管理者がコンサルタントへプラットフォーム手数料と振込手数料を指し引いて出金したことを確認したときに削除される。
             * 管理者が、ユーザーから苦情を受け、客観的な証拠を確認し、返金した後に削除される。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             *
             * sender_nameは入金時に確認できた振込依頼人（身分情報にある姓名）のこと。
             * 身分情報にある姓名はユーザーによって更新が可能なので確認時の情報は変更されても残るように別途保管しておく。
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.awaiting_withdrawal (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL,
                  sender_name TEXT NOT NULL,
                  payment_confirmed_by ccs_schema.email_address NOT NULL,
                  created_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.awaiting_withdrawal To admin_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_withdrawal_user_account_id_idx ON ccs_schema.awaiting_withdrawal (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_withdrawal_consultant_id_idx ON ccs_schema.awaiting_withdrawal (consultant_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_withdrawal_meeting_at_idx ON ccs_schema.awaiting_withdrawal (meeting_at);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX awaiting_withdrawal_created_at_idx ON ccs_schema.awaiting_withdrawal (created_at);",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 管理者がコンサルタントへの出金が出来なかったときに生成される（具体的には出金時に既に口座情報が削除されている場合）サービスの運用期間を通じて存在し続ける。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.left_awaiting_withdrawal (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL,
                  sender_name TEXT NOT NULL,
                  confirmed_by ccs_schema.email_address NOT NULL,
                  created_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ =
            conn.execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.left_awaiting_withdrawal To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX left_awaiting_withdrawal_user_account_id_idx ON ccs_schema.left_awaiting_withdrawal (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX left_awaiting_withdrawal_consultant_id_idx ON ccs_schema.left_awaiting_withdrawal (consultant_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX left_awaiting_withdrawal_meeting_at_idx ON ccs_schema.left_awaiting_withdrawal (meeting_at);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX left_awaiting_withdrawal_created_at_idx ON ccs_schema.left_awaiting_withdrawal (created_at);",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 管理者が相談日時までにユーザーの入金を確認できなかったとき生成される。サービスの運用期間を通じて存在し続ける。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.neglected_payment (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL,
                  neglect_confirmed_by ccs_schema.email_address NOT NULL,
                  created_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.neglected_payment To admin_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX neglected_payment_user_account_id_idx ON ccs_schema.neglected_payment (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX neglected_payment_consultant_id_idx ON ccs_schema.neglected_payment (consultant_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX neglected_payment_meeting_at_idx ON ccs_schema.neglected_payment (meeting_at);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX neglected_payment_created_at_idx ON ccs_schema.neglected_payment (created_at);",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 管理者がコンサルタントへプラットフォーム手数料と振込手数料を指し引いて出金したことを確認した後に生成される。
             * サービスの運用期間を通じて存在し続ける。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             *
             * 口座情報はユーザーによって更新が可能なので、確認時の情報は変更されても残るように別途保管しておく。
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.receipt_of_consultation (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL,
                  platform_fee_rate_in_percentage TEXT NOT NULL,
                  transfer_fee_in_yen INTEGER NOT NULL,
                  reward INTEGER NOT NULL,
                  sender_name TEXT NOT NULL,
                  bank_code TEXT NOT NULL,
                  branch_code TEXT NOT NULL,
                  account_type TEXT NOT NULL,
                  account_number TEXT NOT NULL,
                  account_holder_name TEXT NOT NULL,
                  withdrawal_confirmed_by ccs_schema.email_address NOT NULL,
                  created_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"GRANT SELECT, INSERT ON ccs_schema.receipt_of_consultation To admin_app;",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX receipt_of_consultation_user_account_id_idx ON ccs_schema.receipt_of_consultation (user_account_id);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX receipt_of_consultation_consultant_id_idx ON ccs_schema.receipt_of_consultation (consultant_id);",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX receipt_of_consultation_meeting_at_idx ON ccs_schema.receipt_of_consultation (meeting_at);",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"CREATE INDEX receipt_of_consultation_created_at_idx ON ccs_schema.receipt_of_consultation (created_at);",
                ),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /*
             * 1. 管理者が、ユーザーが相談日時までに入金したにも関わらず支払いの確認を出来なかった場合、返金した後生成される。
             * 2. 管理者が、ユーザーから苦情を受け、客観的な証拠を確認し、返金した後に生成される。
             * (振込手数料は、1の場合管理者が負担し、2の場合はコンサルタントに負担させる（次回コンサルタントに入金する際に損害を差し引く）)
             * サービスの運用期間を通じて存在し続ける。
             *
             * user_account_id、consultant_id、meeting_atは非正規化し、consultationと同じ値を保持する。
             * それらがないと仮定し、結合を使う場合、下記の点で問題があるため非正規化することしている。
             *  - このテーブルをconsultationと結合したとき、条件でのフィルタリングと取得件数制限の処理を同時に正しく処理する方法が煩雑
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.refunded_payment (
                  consultation_id BIGINT PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  consultant_id BIGINT NOT NULL,
                  meeting_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  fee_per_hour_in_yen INTEGER NOT NULL,
                  transfer_fee_in_yen INTEGER NOT NULL,
                  sender_name TEXT NOT NULL,
                  reason TEXT NOT NULL,
                  refund_confirmed_by ccs_schema.email_address NOT NULL,
                  created_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.refunded_payment To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX refunded_payment_user_account_id_idx ON ccs_schema.refunded_payment (user_account_id);",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX refunded_payment_consultant_id_idx ON ccs_schema.refunded_payment (consultant_id);",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"CREATE INDEX refunded_payment_meeting_at_idx ON ccs_schema.refunded_payment (meeting_at);"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"CREATE INDEX refunded_payment_created_at_idx ON ccs_schema.refunded_payment (created_at);"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが管理者に新規に身分確認を依頼したときに生成される。
             * 管理者が身分確認依頼を承認、または拒否したときに削除される。
             */
            /*
             * user_account一つに対して、create_identity_req (本人確認依頼 (新規)) は0もしくは1の関係とする。従って、user_account_idをPRIMARY KEYに指定する
             * 画像ファイルの実体は、データベース外に保存している。user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 画像の実体との紐づけが知らないうちに解除される可能性がある。そのため、user_account_idは外部キーとしない
             */
            /*
             * image1_file_name_without_ext, image2_file_name_without_ext
             * 画像ファイル名は、user_account_idと組み合わせて外部に保存する。そのため、データベースに保管する値のUNIQUE指定は必須ではない
             * UNIQUEにしたときのNULLの扱いがデータベースごとに異なる可能性がある。従って、その点も考慮し、NULL利用があるカラムにUNIQUE付与を避ける
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.create_identity_req (
                  user_account_id BIGINT PRIMARY KEY,
                  last_name VARCHAR (64) NOT NULL,
                  first_name VARCHAR (64) NOT NULL,
                  last_name_furigana VARCHAR (64) NOT NULL,
                  first_name_furigana VARCHAR (64) NOT NULL,
                  date_of_birth DATE NOT NULL,
                  prefecture VARCHAR (4) NOT NULL,
                  city VARCHAR (32) NOT NULL,
                  address_line1 VARCHAR (128) NOT NULL,
                  address_line2 VARCHAR (128),
                  telephone_number VARCHAR (13) NOT NULL,
                  image1_file_name_without_ext ccs_schema.uuid_simple_form NOT NULL,
                  image2_file_name_without_ext ccs_schema.uuid_simple_form,
                  requested_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.create_identity_req To user_app;"),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.create_identity_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX create_identity_req_requested_at_idx ON ccs_schema.create_identity_req (requested_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がユーザーの新規身分確認依頼を承認したときに生成される。サービスの運用期間を通じて存在し続ける */
            /*
             * user_accountのuser_account_idはBIGSERIALなので重複する心配はない。そのため、
             * approved_create_identity_req (本人確認依頼 (新規) の承認) のPRIMARY KEYとしてuser_account_idを利用する
             */
            /*
             * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 管理者の把握しないうちに承認した記録が消去される可能性がある。そのため、user_account_idは外部キーとしない
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.approved_create_identity_req (
                  user_account_id BIGINT PRIMARY KEY,
                  last_name VARCHAR (64) NOT NULL,
                  first_name VARCHAR (64) NOT NULL,
                  last_name_furigana VARCHAR (64) NOT NULL,
                  first_name_furigana VARCHAR (64) NOT NULL,
                  date_of_birth DATE NOT NULL,
                  prefecture VARCHAR (4) NOT NULL,
                  city VARCHAR (32) NOT NULL,
                  address_line1 VARCHAR (128) NOT NULL,
                  address_line2 VARCHAR (128),
                  telephone_number VARCHAR (13) NOT NULL,
                  image1_file_name_without_ext ccs_schema.uuid_simple_form NOT NULL,
                  image2_file_name_without_ext ccs_schema.uuid_simple_form,
                  approved_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  approved_by ccs_schema.email_address NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.approved_create_identity_req To admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がユーザーの新規身分確認依頼を拒否したときに生成される。サービスの運用期間を通じて存在し続ける */
            /*
             * 複数回拒否の記録が残る可能性があるため、user_accountのuser_account_idをPRIMARY KEYとしては扱わない。
             */
            /*
             * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 管理者の把握しないうちに拒否した記録が消去される可能性がある。そのため、user_account_idは外部キーとしない
             */
            /*
             * アップロードされた画像は拒否時に削除するため、
             * image1_file_name_without_ext, image2_file_name_without_extは保持させない。
             */
            /*
             * PRIMARY KEYはSEQUENCE名にしたときに識別子の63文字制限に引っかからないように命名する（rjd_cre_identity_id）
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.rejected_create_identity_req (
                  rjd_cre_identity_id BIGSERIAL PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  last_name VARCHAR (64) NOT NULL,
                  first_name VARCHAR (64) NOT NULL,
                  last_name_furigana VARCHAR (64) NOT NULL,
                  first_name_furigana VARCHAR (64) NOT NULL,
                  date_of_birth DATE NOT NULL,
                  prefecture VARCHAR (4) NOT NULL,
                  city VARCHAR (32) NOT NULL,
                  address_line1 VARCHAR (128) NOT NULL,
                  address_line2 VARCHAR (128),
                  telephone_number VARCHAR (13) NOT NULL,
                  reason VARCHAR (256) NOT NULL,
                  rejected_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  rejected_by ccs_schema.email_address NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.rejected_create_identity_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.rejected_create_identity_req_rjd_cre_identity_id_seq TO admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* ユーザーが管理者に身分情報の更新を依頼したときに生成される。
             * 管理者が身分情報の更新依頼を承認、または拒否したときに削除される。
             */
            /*
             * user_account一つに対して、update_identity_req (本人確認依頼 (更新)) は0もしくは1の関係とする。従って、user_account_idをPRIMARY KEYに指定する
             * 画像ファイルの実体は、データベース外に保存している。user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 画像の実体との紐づけが知らないうちに解除される可能性がある。そのため、user_account_idは外部キーとしない
             */
            /*
             * image1_file_name_without_ext, image2_file_name_without_ext
             * 画像ファイル名は、user_account_idと組み合わせて外部に保存する。そのため、データベースに保管する値のUNIQUE指定は必須ではない
             * UNIQUEにしたときのNULLの扱いがデータベースごとに異なる可能性がある。従って、その点も考慮し、NULL利用があるカラムにUNIQUE付与を避ける
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.update_identity_req (
                    user_account_id BIGINT PRIMARY KEY,
                    last_name VARCHAR (64) NOT NULL,
                    first_name VARCHAR (64) NOT NULL,
                    last_name_furigana VARCHAR (64) NOT NULL,
                    first_name_furigana VARCHAR (64) NOT NULL,
                    date_of_birth DATE NOT NULL,
                    prefecture VARCHAR (4) NOT NULL,
                    city VARCHAR (32) NOT NULL,
                    address_line1 VARCHAR (128) NOT NULL,
                    address_line2 VARCHAR (128),
                    telephone_number VARCHAR (13) NOT NULL,
                    image1_file_name_without_ext ccs_schema.uuid_simple_form NOT NULL,
                    image2_file_name_without_ext ccs_schema.uuid_simple_form,
                    requested_at TIMESTAMP WITH TIME ZONE NOT NULL
                  );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.update_identity_req To user_app;"),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.update_identity_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX update_identity_req_requested_at_idx ON ccs_schema.update_identity_req (requested_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がユーザーの身分情報の更新依頼を承認したときに生成される。サービスの運用期間を通じて存在し続ける */
            /*
             * 複数回更新の記録が残る可能性があるため、user_accountのuser_account_idをPRIMARY KEYとしては扱わない。
             */
            /*
             * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 管理者の把握しないうちに承認した記録が消去される可能性がある。そのため、user_account_idは外部キーとしない
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.approved_update_identity_req (
                  appr_upd_identity_req_id BIGSERIAL PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  last_name VARCHAR (64) NOT NULL,
                  first_name VARCHAR (64) NOT NULL,
                  last_name_furigana VARCHAR (64) NOT NULL,
                  first_name_furigana VARCHAR (64) NOT NULL,
                  date_of_birth DATE NOT NULL,
                  prefecture VARCHAR (4) NOT NULL,
                  city VARCHAR (32) NOT NULL,
                  address_line1 VARCHAR (128) NOT NULL,
                  address_line2 VARCHAR (128),
                  telephone_number VARCHAR (13) NOT NULL,
                  image1_file_name_without_ext ccs_schema.uuid_simple_form NOT NULL,
                  image2_file_name_without_ext ccs_schema.uuid_simple_form,
                  approved_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  approved_by ccs_schema.email_address NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.approved_update_identity_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.approved_update_identity_req_appr_upd_identity_req_id_seq TO admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がユーザーの身分情報の更新依頼を拒否したときに生成される。サービスの運用期間を通じて存在し続ける */
            /*
             * 複数回拒否の記録が残る可能性があるため、user_accountのuser_account_idをPRIMARY KEYとしては扱わない。
             */
            /*
             * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 管理者の把握しないうちに拒否した記録が消去される可能性がある。そのため、user_account_idは外部キーとしない
             */
            /*
             * アップロードされた画像は拒否時に削除するため、
             * image1_file_name_without_ext, image2_file_name_without_extは保持させない。
             */
            /*
             * PRIMARY KEYはSEQUENCE名にしたときに識別子の63文字制限に引っかからないように命名する（rjd_upd_identity_id）
             */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.rejected_update_identity_req (
                  rjd_upd_identity_id BIGSERIAL PRIMARY KEY,
                  user_account_id BIGINT NOT NULL,
                  last_name VARCHAR (64) NOT NULL,
                  first_name VARCHAR (64) NOT NULL,
                  last_name_furigana VARCHAR (64) NOT NULL,
                  first_name_furigana VARCHAR (64) NOT NULL,
                  date_of_birth DATE NOT NULL,
                  prefecture VARCHAR (4) NOT NULL,
                  city VARCHAR (32) NOT NULL,
                  address_line1 VARCHAR (128) NOT NULL,
                  address_line2 VARCHAR (128),
                  telephone_number VARCHAR (13) NOT NULL,
                  reason VARCHAR (256) NOT NULL,
                  rejected_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  rejected_by ccs_schema.email_address NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.rejected_update_identity_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.rejected_update_identity_req_rjd_upd_identity_id_seq TO admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            .execute(
                /* ユーザーが管理者に職務経歴の確認を依頼したときに生成される。
                 * 管理者が職務経歴の内容を承認、または拒否したときに削除される。
                 */
                /*
                 * 最大MAX_NUM_OF_CAREER_PER_USER_ACCOUNT回のリクエストを受け付け可能にするため、
                 * user_accountのuser_account_idをUNIQUEとしては扱わない。
                 */
                /*
                 * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
                 * 管理者の把握しないうちにレコードが消去される可能性がある。そのため、user_account_idは外部キーとしない
                 */
                sql.stmt(
                    r"CREATE TABLE ccs_schema.create_career_req (
                    create_career_req_id BIGSERIAL PRIMARY KEY,
                    user_account_id BIGINT NOT NULL,
                    company_name VARCHAR (256) NOT NULL,
                    department_name VARCHAR (256),
                    office VARCHAR (256),
                    career_start_date DATE NOT NULL,
                    career_end_date DATE,
                    contract_type ccs_schema.contract_type NOT NULL,
                    profession VARCHAR (128),
                    annual_income_in_man_yen INTEGER,
                    is_manager BOOLEAN NOT NULL,
                    position_name VARCHAR (128),
                    is_new_graduate BOOLEAN NOT NULL,
                    note VARCHAR (2048),
                    image1_file_name_without_ext ccs_schema.uuid_simple_form NOT NULL,
                    image2_file_name_without_ext ccs_schema.uuid_simple_form,
                    requested_at TIMESTAMP WITH TIME ZONE NOT NULL
                  );",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT ON ccs_schema.create_career_req To user_app;"))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, UPDATE, DELETE ON ccs_schema.create_career_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT USAGE ON SEQUENCE ccs_schema.create_career_req_create_career_req_id_seq TO user_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX create_career_req_requested_at_idx ON ccs_schema.create_career_req (requested_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            .execute(
                /* 管理者がユーザーの職務経歴の内容を承認したときに生成される。サービスの運用期間を通じて存在し続ける */
                /*
                 * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
                 * 管理者の把握しないうちにレコードが消去される可能性がある。そのため、user_account_idは外部キーとしない
                 */
                sql.stmt(
                    r"CREATE TABLE ccs_schema.approved_create_career_req (
                    appr_cre_career_req_id BIGSERIAL PRIMARY KEY,
                    user_account_id BIGINT NOT NULL,
                    company_name VARCHAR (256) NOT NULL,
                    department_name VARCHAR (256),
                    office VARCHAR (256),
                    career_start_date DATE NOT NULL,
                    career_end_date DATE,
                    contract_type ccs_schema.contract_type NOT NULL,
                    profession VARCHAR (128),
                    annual_income_in_man_yen INTEGER,
                    is_manager BOOLEAN NOT NULL,
                    position_name VARCHAR (128),
                    is_new_graduate BOOLEAN NOT NULL,
                    note VARCHAR (2048),
                    image1_file_name_without_ext ccs_schema.uuid_simple_form NOT NULL,
                    image2_file_name_without_ext ccs_schema.uuid_simple_form,
                    approved_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    approved_by ccs_schema.email_address NOT NULL
                  );",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.approved_create_career_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT USAGE ON SEQUENCE ccs_schema.approved_create_career_req_appr_cre_career_req_id_seq TO admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* 下記のタイミングで生成される
             *   - 以下の3つの内、いずれかが実行されたとき
             *     1. 管理者がそのユーザーの職務経歴の内容を（初めて）承認したとき
             *     2. ユーザーが相談料を（初めて）設定したとき
             *     3. ユーザーが入金口座を（初めて）設定したとき
             *   - 管理者がユーザーを有効化したとき（同時に検索用インデックスにデータを生成して投入する）
             *
             * 下記のタイミングで削除される（削除時に同時に検索用インデックスからもデータを削除する）
             * 　- ユーザーがアカウントを削除したとき（削除されたユーザーテーブルに移されたとき）
             *   - 管理者がユーザーを無効化したとき
             */
            /* user_account一つに対して、document（検索用の情報）は0もしくは1の関係とする。 */
            /* document_idにはuser_accountと同じ値をセットする。*/
            /* document_idがある場合、インデックスに検索用の情報がある。ない場合、インデックスに検索用の情報が存在しない */
            .execute(sql.stmt(r"CREATE TABLE ccs_schema.document (
              user_account_id BIGINT PRIMARY KEY,
              document_id BIGINT NOT NULL
            );"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(
                    r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.document To user_app;",
                ),
            )
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ =
            conn.execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.document To admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            .execute(
                /* 管理者がユーザーの職務経歴の内容を拒否したときに生成される。サービスの運用期間を通じて存在し続ける */
                /*
                 * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
                 * 管理者の把握しないうちにレコードが消去される可能性がある。そのため、user_account_idは外部キーとしない
                 */
                /*
                 * アップロードされた画像は拒否時に削除するため、
                 * image1_file_name_without_ext, image2_file_name_without_extは保持させない。
                 */
                sql.stmt(
                    r"CREATE TABLE ccs_schema.rejected_create_career_req (
                    rjd_cre_career_req_id BIGSERIAL PRIMARY KEY,
                    user_account_id BIGINT NOT NULL,
                    company_name VARCHAR (256) NOT NULL,
                    department_name VARCHAR (256),
                    office VARCHAR (256),
                    career_start_date DATE NOT NULL,
                    career_end_date DATE,
                    contract_type ccs_schema.contract_type NOT NULL,
                    profession VARCHAR (128),
                    annual_income_in_man_yen INTEGER,
                    is_manager BOOLEAN NOT NULL,
                    position_name VARCHAR (128),
                    is_new_graduate BOOLEAN NOT NULL,
                    note VARCHAR (2048),
                    reason VARCHAR (256) NOT NULL,
                    rejected_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    rejected_by ccs_schema.email_address NOT NULL
                  );",
                ),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT ON ccs_schema.rejected_create_career_req To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT USAGE ON SEQUENCE ccs_schema.rejected_create_career_req_rjd_cre_career_req_id_seq TO admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がメンテナンス期間を設定したときに生成される。
             * サービスの運用期間を通じて存在し続ける。不要なデータはメンテナンス日時でフィルタリングする */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.maintenance (
                  maintenance_id BIGSERIAL PRIMARY KEY,
                  maintenance_start_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  maintenance_end_at TIMESTAMP WITH TIME ZONE NOT NULL,
                  CHECK (maintenance_end_at > maintenance_start_at)
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT ON ccs_schema.maintenance To user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.maintenance To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.maintenance_maintenance_id_seq TO admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX maintenance_maintenance_end_at_idx ON ccs_schema.maintenance (maintenance_end_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* 管理者がお知らせを作成したときに生成される。
             * サービスの運用期間を通じて存在し続ける。不要なデータは掲載日時でフィルタリングする */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.news (
                  news_id BIGSERIAL PRIMARY KEY,
                  title TEXT NOT NULL,
                  body TEXT NOT NULL,
                  published_at TIMESTAMP WITH TIME ZONE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT ON ccs_schema.news To user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.news To admin_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT USAGE ON SEQUENCE ccs_schema.news_news_id_seq TO admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"CREATE INDEX news_published_at_idx ON ccs_schema.news (published_at);"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* サービスのオーナーが管理者を作成したときに生成される。
             * サービスのオーナーが管理者を削除したときに削除される。 */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.admin_account (
                    admin_account_id BIGSERIAL PRIMARY KEY,
                    email_address ccs_schema.email_address NOT NULL UNIQUE,
                    hashed_password BYTEA NOT NULL,
                    last_login_time TIMESTAMP WITH TIME ZONE,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    mfa_enabled_at TIMESTAMP WITH TIME ZONE
                  );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.admin_account To admin_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT USAGE ON SEQUENCE ccs_schema.admin_account_admin_account_id_seq TO admin_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* サービスのオーナーが二段階認証を有効にしたときに生成される。
             * サービスのオーナーが二段階認証を無効にしたときに削除される。
             * サービスのオーナーが管理者を削除したときに削除される。
             */
            /* 管理者向けにはリカバリーコードでのログインは提供しない */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.admin_mfa_info (
                admin_account_id BIGINT PRIMARY KEY,
                base32_encoded_secret TEXT NOT NULL
              );",
            ))
            .await
            .map(|_| ())?;
        // 定期削除ツールはadmin_appのロールを使う。そのため、定期削除ツールが削除できるようにDELETE権限を保持させる
        let _ = conn
            .execute(
                sql.stmt(
                    r"GRANT SELECT, INSERT, DELETE ON ccs_schema.admin_mfa_info To admin_app;",
                ),
            )
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let db_backend = manager.get_database_backend();
        let sql = Sql::new(db_backend);

        let _ = conn
            .execute(sql.stmt(r"DROP SCHEMA ccs_schema CASCADE;"))
            .await
            .map(|_| ())?;
        Ok(())
    }
}

struct Sql {
    db_backend: DatabaseBackend,
}

impl Sql {
    fn new(db_backend: DatabaseBackend) -> Self {
        Self { db_backend }
    }
    fn stmt(&self, sql: &str) -> Statement {
        Statement::from_string(self.db_backend, sql.to_owned())
    }
}
