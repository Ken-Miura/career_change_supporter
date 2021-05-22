/* TODO: パスワード変更＋管理方法の検討 */
CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';
CREATE ROLE advisor_app WITH LOGIN PASSWORD 'test5678';

CREATE SCHEMA career_change_supporter_schema;
GRANT USAGE ON SCHEMA career_change_supporter_schema TO user_app;
GRANT USAGE ON SCHEMA career_change_supporter_schema TO advisor_app;

CREATE DOMAIN career_change_supporter_schema.email_address AS VARCHAR (254) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
/* simpleフォーム (半角英数字32文字。ハイフン、波括弧を含まない) での入出力を行いたいので、標準のUUID型を使わない */
CREATE DOMAIN career_change_supporter_schema.uuid_simple_form AS CHAR (32) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9]+$' );

/* data structure for user */
CREATE TABLE career_change_supporter_schema.user_account (
  user_account_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address career_change_supporter_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE
);
GRANT SELECT, INSERT, UPDATE, DELETE ON career_change_supporter_schema.user_account To user_app;
/* 
 * NOTE: 下記の参考によると、SERIALで暗黙的に作成されるSEQUENCEはtablename_colname_seqで定められる
 * 参考: https://www.postgresql.org/docs/13/datatype-numeric.html#DATATYPE-SERIAL
 */
GRANT USAGE ON SEQUENCE career_change_supporter_schema.user_account_user_account_id_seq TO user_app;

CREATE TABLE career_change_supporter_schema.user_temporary_account (
  user_temporary_account_id career_change_supporter_schema.uuid_simple_form PRIMARY KEY,
  /* 一度仮登録した後、それを忘れてしまいもう一度仮登録したいケースを考え、UNIQUEにしない。user_temporary_account_idがPRIMARY KEYなので一意に検索は可能 */
  email_address career_change_supporter_schema.email_address,
  hashed_password BYTEA NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, DELETE ON career_change_supporter_schema.user_temporary_account To user_app;

/* data structure for advisor */
CREATE TABLE career_change_supporter_schema.advisor_registration_request (
  advisor_registration_request_id career_change_supporter_schema.uuid_simple_form PRIMARY KEY,
  /* 一度登録依頼を出した後、それを忘れてしまいもう一度登録依頼したいケースを考え、UNIQUEにしない。advisor_registration_request_idがPRIMARY KEYなので一意に検索は可能 */
  email_address career_change_supporter_schema.email_address,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, DELETE ON career_change_supporter_schema.advisor_registration_request To advisor_app;

/* TODO: 必要情報のアップロード処理が追加され次第、データを追加して更新する */
CREATE TABLE career_change_supporter_schema.advisor_account (
  advisor_account_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address career_change_supporter_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE
);
GRANT SELECT, INSERT, UPDATE, DELETE ON career_change_supporter_schema.advisor_account To advisor_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.advisor_account_advisor_account_id_seq TO advisor_app;
