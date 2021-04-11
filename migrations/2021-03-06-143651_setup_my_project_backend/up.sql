CREATE SCHEMA my_project_schema;
CREATE DOMAIN my_project_schema.email_address AS VARCHAR (254) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
/* simpleフォーム (半角英数字32文字。ハイフン、波括弧を含まない) での入出力を行いたいので、標準のUUID型を使わない */
CREATE DOMAIN my_project_schema.uuid_simple_form AS CHAR (32) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9]+$' );
CREATE TABLE my_project_schema.user_account (
  id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address my_project_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE
);
CREATE TABLE my_project_schema.user_temporary_account (
  id SERIAL PRIMARY KEY,
  /* NOTE: query_idがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  temporary_account_id my_project_schema.uuid_simple_form UNIQUE,
  /* 一度仮登録した後、それを忘れてしまいもう一度仮登録したいケースを考え、UNIQUEにしない。query_idがUNIQUEなので一意に検索は可能 */
  email_address my_project_schema.email_address,
  hashed_password BYTEA NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
