// Copyright 2021 Ken Miura

use entity::sea_orm::{DatabaseBackend, Statement};
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
            /* ccs = Career Change Supporter */
            .execute(sql.stmt(r"CREATE SCHEMA ccs_schema;"))
            .await
            .map(|_| ())?;
        // ROLE定義
        // TODO: ROLEのパスワード変更＋管理方法の検討
        let _ = conn
            /*  */
            .execute(sql.stmt(r"CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT USAGE ON SCHEMA ccs_schema TO user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"CREATE ROLE admin_app WITH LOGIN PASSWORD 'test13579';"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT USAGE ON SCHEMA ccs_schema TO admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"CREATE ROLE admin_account_app WITH LOGIN PASSWORD 'test24680';"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"GRANT USAGE ON SCHEMA ccs_schema TO admin_account_app;"))
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
            user_account_id SERIAL PRIMARY KEY,
            email_address ccs_schema.email_address NOT NULL UNIQUE,
            hashed_password BYTEA NOT NULL,
            last_login_time TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL
          );"))
        .await
        .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(
                r"GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.user_account To user_app;",
            ))
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
        let _ = conn
            .execute(sql.stmt(r"DROP ROLE user_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"DROP ROLE admin_app;"))
            .await
            .map(|_| ())?;
        let _ = conn
            .execute(sql.stmt(r"DROP ROLE admin_account_app;"))
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
