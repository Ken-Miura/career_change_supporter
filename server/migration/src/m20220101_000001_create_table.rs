// Copyright 2021 Ken Miura

use entity::sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_schema::migration::*;

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
            /* PAY.JPより回答してもらった仕様をそのままチェック */
            .execute(sql.stmt(r"CREATE DOMAIN ccs_schema.tenant_id AS VARCHAR (100) CHECK ( VALUE ~ '^[-_0-9a-zA-Z]{1,100}$' );"))
            .await
            .map(|_| ())?;
        let _ = conn
            /* 
             * regular: 正社員、contract: 契約社員、other: その他
             * TODO: enumがサポートされた後、修正する
             */
            .execute(sql.stmt(r"CREATE DOMAIN ccs_schema.contract_type AS VARCHAR (8) CHECK (VALUE ~ 'regular' OR VALUE ~ 'contract' OR VALUE ~ 'other');"))
            .await
            .map(|_| ())?;
        // その他（TABLE、INDEX等）の定義
        let _ = conn
        /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
        .execute(sql.stmt(r"CREATE TABLE ccs_schema.user_account (
            user_account_id BIGSERIAL PRIMARY KEY,
            email_address ccs_schema.email_address NOT NULL UNIQUE,
            hashed_password BYTEA NOT NULL,
            last_login_time TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
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
            .execute(sql.stmt(
                r"GRANT UPDATE (hashed_password, last_login_time) ON ccs_schema.user_account To user_app;",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT ON ccs_schema.user_account To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT UPDATE (disabled_at) ON ccs_schema.user_account To admin_app;"),
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

        let _ = conn
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

        let _ = conn
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

        let _ = conn
            /* user_account一つに対して、identityは0もしくは1の関係とする。従って、user_account_idを外部キーかつ主キーとする */
            /* prefecture => 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
            /* city => 市区町村の最大文字数は6文字。しかし、市区町村は頻繁に名前が変更される可能性があるので長さに余裕をもたせる */
            /*
             * telephone_number
             * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
             * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
             */
            .execute(sql.stmt(r"CREATE TABLE ccs_schema.identity (
              user_account_id BIGINT PRIMARY KEY REFERENCES ccs_schema.user_account(user_account_id) ON DELETE CASCADE ON UPDATE RESTRICT,
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
            /* アカウント削除はユーザー自身が行う。そのため削除権限はユーザー (user_app) に付与する */
            .execute(sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.identity To user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT, UPDATE ON ccs_schema.identity To admin_app;"))
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
                /* annual_income_in_man_yen => 万円単位での年収 */
                sql.stmt(r"CREATE TABLE ccs_schema.career (
                    career_id BIGSERIAL PRIMARY KEY,
                    user_account_id BIGINT NOT NULL REFERENCES ccs_schema.user_account(user_account_id) ON DELETE CASCADE ON UPDATE RESTRICT,
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
                  );"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                /* 職務経歴は、管理者 (admin_app) が提出されたエビデンスを確認し、レコードを挿入、更新する。従って、ユーザー (user_app) には挿入、更新権限は持たせない。*/
                /* アカウント削除はユーザー自身が行う。そのため削除権限はユーザー (user_app) に付与する */
                sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.career To user_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT SELECT, INSERT, UPDATE ON ccs_schema.career To admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(
                sql.stmt(r"GRANT USAGE ON SEQUENCE ccs_schema.career_career_id_seq TO admin_app;"),
            )
            .await
            .map(|_| ())?;

        let _ = conn
            /* user_account一つに対して、consulting_feeは0もしくは1の関係とする。従って、user_account_idを外部キーかつ主キーとして扱う */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.consulting_fee (
                  user_account_id BIGINT PRIMARY KEY REFERENCES ccs_schema.user_account(user_account_id) ON DELETE CASCADE ON UPDATE RESTRICT,
                  fee_per_hour_in_yen INTEGER NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.consulting_fee To user_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            /* user_account一つに対して、tenantは0もしくは1の関係とする。従って、user_account_idを外部キーかつ主キーとして扱う */
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.tenant (
                  user_account_id BIGINT PRIMARY KEY REFERENCES ccs_schema.user_account(user_account_id) ON DELETE CASCADE ON UPDATE RESTRICT,
                  tenant_id ccs_schema.tenant_id UNIQUE NOT NULL
                );",
            ))
            .await
            .map(|_| ())?;
        let _ = conn
            /* 一度作成したtenant_idは、payjpとの連携に必要となる。ユーザーに同じtenant_idを利用することを強制するため、UPDATEは許可しない */
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, DELETE ON ccs_schema.tenant To user_app;",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
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
            /*
             * 複数回拒否の記録が残る可能性があるため、user_accountのuser_account_idをPRIMARY KEYとしては扱わない。
             */
            /*
             * user_account_idを外部キーにすると、user_accountの操作時に同時にこちらのテーブルのレコードも操作されて、
             * 管理者の把握しないうちに拒否した記録が消去される可能性がある。そのため、user_account_idは外部キーとしない
             */
            /*
             * 拒否した場合、アップロードされた画像は削除するため、image1_file_name_without_ext, image2_file_name_without_extは保持しない。
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
                  reason VARCHAR (512) NOT NULL,
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
        let _ = conn
            .execute(
                sql.stmt(r"GRANT SELECT, DELETE ON ccs_schema.update_identity_req To admin_app;"),
            )
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"CREATE INDEX update_identity_req_requested_at_idx ON ccs_schema.update_identity_req (requested_at);",
            ))
            .await
            .map(|_| ())?;

        let _ = conn
            .execute(sql.stmt(
                r"CREATE TABLE ccs_schema.admin_account (
                    admin_account_id BIGSERIAL PRIMARY KEY,
                    email_address ccs_schema.email_address NOT NULL UNIQUE,
                    hashed_password BYTEA NOT NULL,
                    last_login_time TIMESTAMP WITH TIME ZONE
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
