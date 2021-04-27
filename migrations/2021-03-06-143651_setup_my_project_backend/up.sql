/* TODO: パスワード変更＋管理方法の検討 */
CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';

CREATE SCHEMA my_project_schema;
GRANT USAGE ON SCHEMA my_project_schema TO user_app;

CREATE DOMAIN my_project_schema.email_address AS VARCHAR (254) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
/* simpleフォーム (半角英数字32文字。ハイフン、波括弧を含まない) での入出力を行いたいので、標準のUUID型を使わない */
CREATE DOMAIN my_project_schema.uuid_simple_form AS CHAR (32) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9]+$' );

CREATE TABLE my_project_schema.user_account (
  user_account_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address my_project_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE
);
GRANT SELECT, INSERT, UPDATE, DELETE ON my_project_schema.user_account To user_app;
/* 
 * NOTE: 下記の参考によると、SERIALで暗黙的に作成されるSEQUENCEはtablename_colname_seqで定められる
 * 参考: https://www.postgresql.org/docs/13/datatype-numeric.html#DATATYPE-SERIAL
 */
GRANT USAGE ON SEQUENCE my_project_schema.user_account_user_account_id_seq TO user_app;

CREATE TABLE my_project_schema.user_temporary_account (
  user_temporary_account_id my_project_schema.uuid_simple_form PRIMARY KEY,
  /* 一度仮登録した後、それを忘れてしまいもう一度仮登録したいケースを考え、UNIQUEにしない。user_temporary_account_idがPRIMARY KEYなので一意に検索は可能 */
  email_address my_project_schema.email_address,
  hashed_password BYTEA NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, UPDATE, DELETE ON my_project_schema.user_temporary_account To user_app;