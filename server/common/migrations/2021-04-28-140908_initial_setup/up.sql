/* TODO: パスワード変更＋管理方法の検討 */
CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';
CREATE ROLE admin_app WITH LOGIN PASSWORD 'test13579';
CREATE ROLE admin_account_app WITH LOGIN PASSWORD 'test24680';

/* ccs = Career Change Supporter */
CREATE SCHEMA ccs_schema;
GRANT USAGE ON SCHEMA ccs_schema TO user_app;
GRANT USAGE ON SCHEMA ccs_schema TO admin_app;
GRANT USAGE ON SCHEMA ccs_schema TO admin_account_app;

/* TODO: dieselでenumがサポートされた後に採用する
   CREATE TYPE ccs_schema.sex_enum AS ENUM ('male', 'female');
 */
CREATE DOMAIN ccs_schema.sex AS VARCHAR (6) NOT NULL CHECK (VALUE ~ 'male' OR VALUE ~ 'female');
CREATE DOMAIN ccs_schema.email_address AS VARCHAR (254) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
/* simpleフォーム (半角英数字32文字。ハイフン、波括弧を含まない) での入出力を行いたいので、標準のUUID型を使わない */
CREATE DOMAIN ccs_schema.uuid_simple_form AS CHAR (32) CHECK ( VALUE ~ '^[a-zA-Z0-9]+$' );
/* PAY.JPより回答してもらった仕様をそのままチェック */
CREATE DOMAIN ccs_schema.tenant_id AS VARCHAR (100) CHECK ( VALUE ~ '^[-_0-9a-zA-Z]{1,100}$' );

CREATE TABLE ccs_schema.user_account (
  user_account_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address ccs_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.user_account To user_app;
/* 
 * NOTE: 下記の参考によると、SERIALで暗黙的に作成されるSEQUENCEはtablename_colname_seqで定められる
 * 参考: https://www.postgresql.org/docs/13/datatype-numeric.html#DATATYPE-SERIAL
 */
GRANT USAGE ON SEQUENCE ccs_schema.user_account_user_account_id_seq TO user_app;

CREATE TABLE ccs_schema.user_temp_account (
  user_temp_account_id ccs_schema.uuid_simple_form PRIMARY KEY,
  /* 一度仮登録した後、それを忘れてしまいもう一度仮登録したいケースを考え、UNIQUEにしない。user_temp_account_idがPRIMARY KEYなので一意に検索は可能 */
  email_address ccs_schema.email_address,
  hashed_password BYTEA NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT ON ccs_schema.user_temp_account To user_app;

/* 
 * ユーザーが利用規約に同意した証拠となる。
 * そのため、後から同意したことを追跡できるように、アカウントが削除されても利用規約の合意は削除されないようにする
 *（user_account_idを外部キーとしてuser_account.user_account_idと関連付けない）
 */
CREATE TABLE ccs_schema.terms_of_use (
  user_account_id INTEGER NOT NULL,
  ver INTEGER NOT NULL,
  email_address ccs_schema.email_address,
  agreed_at TIMESTAMP WITH TIME ZONE NOT NULL,
  PRIMARY KEY (user_account_id, ver)
);
GRANT SELECT, INSERT ON ccs_schema.terms_of_use To user_app;

CREATE TABLE ccs_schema.admin_account (
  admin_account_id SERIAL PRIMARY KEY,
  email_address ccs_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE
);
GRANT SELECT ON ccs_schema.admin_account To admin_app;
GRANT UPDATE (last_login_time) ON ccs_schema.admin_account To admin_app;

GRANT SELECT, INSERT, UPDATE, DELETE ON ccs_schema.admin_account To admin_account_app;
GRANT USAGE ON SEQUENCE ccs_schema.admin_account_admin_account_id_seq TO admin_account_app;
