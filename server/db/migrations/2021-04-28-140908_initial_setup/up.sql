/* TODO: パスワード変更＋管理方法の検討 */
CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';
CREATE ROLE advisor_app WITH LOGIN PASSWORD 'test5678';
CREATE ROLE administrator_app WITH LOGIN PASSWORD 'test13579';
CREATE ROLE administrator_tool_app WITH LOGIN PASSWORD 'test24680';

CREATE SCHEMA career_change_supporter_schema;
GRANT USAGE ON SCHEMA career_change_supporter_schema TO user_app;
GRANT USAGE ON SCHEMA career_change_supporter_schema TO advisor_app;
GRANT USAGE ON SCHEMA career_change_supporter_schema TO administrator_app;
GRANT USAGE ON SCHEMA career_change_supporter_schema TO administrator_tool_app;

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
GRANT SELECT, UPDATE, DELETE ON career_change_supporter_schema.advisor_account To advisor_app;
/* TODO: 単なる読み取りだけの場合もチェックする */
GRANT USAGE ON SEQUENCE career_change_supporter_schema.advisor_account_advisor_account_id_seq TO advisor_app;

GRANT SELECT, INSERT, UPDATE ON career_change_supporter_schema.advisor_account To administrator_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.advisor_account_advisor_account_id_seq TO administrator_app;

CREATE TABLE career_change_supporter_schema.advisor_account_creation_request (
  advisor_acc_request_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address career_change_supporter_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_name VARCHAR (127) NOT NULL,
  first_name VARCHAR (127) NOT NULL,
  last_name_furigana VARCHAR (127) NOT NULL,
  first_name_furigana VARCHAR (127) NOT NULL,
  /*
   * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
   * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
   */
  telephone_number VARCHAR (13) NOT NULL,
  year_of_birth SMALLINT NOT NULL,
  month_of_birth SMALLINT NOT NULL,
  day_of_birth SMALLINT NOT NULL,
  /* 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
  prefecture VARCHAR (4) NOT NULL,
  /* 市区町村の最大文字数は6文字。市区町村は都道府県と比較し、頻繁に名前が変更されるので、長さに余裕をもたせる */
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
  requested_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT INSERT ON career_change_supporter_schema.advisor_account_creation_request To advisor_app;
GRANT SELECT (email_address) ON career_change_supporter_schema.advisor_account_creation_request To advisor_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.advisor_account_creation_request_advisor_acc_request_id_seq TO advisor_app;
GRANT SELECT, DELETE ON career_change_supporter_schema.advisor_account_creation_request To administrator_app;

CREATE TABLE career_change_supporter_schema.administrator_account (
  administrator_account_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address career_change_supporter_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_login_time TIMESTAMP WITH TIME ZONE
);
GRANT SELECT ON career_change_supporter_schema.administrator_account To administrator_app;
GRANT UPDATE (last_login_time) ON career_change_supporter_schema.administrator_account To administrator_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.administrator_account_administrator_account_id_seq TO administrator_app;

GRANT SELECT, INSERT, UPDATE, DELETE ON career_change_supporter_schema.administrator_account To administrator_tool_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.administrator_account_administrator_account_id_seq TO administrator_tool_app;

CREATE TABLE career_change_supporter_schema.advisor_reg_req_approved (
  advisor_reg_req_approved_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address career_change_supporter_schema.email_address UNIQUE,
  last_name VARCHAR (127) NOT NULL,
  first_name VARCHAR (127) NOT NULL,
  last_name_furigana VARCHAR (127) NOT NULL,
  first_name_furigana VARCHAR (127) NOT NULL,
  /*
   * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
   * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
   */
  telephone_number VARCHAR (13) NOT NULL,
  year_of_birth SMALLINT NOT NULL,
  month_of_birth SMALLINT NOT NULL,
  day_of_birth SMALLINT NOT NULL,
  /* 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
  prefecture VARCHAR (4) NOT NULL,
  /* 市区町村の最大文字数は6文字。市区町村は都道府県と比較し、頻繁に名前が変更されるので、長さに余裕をもたせる */
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
  approved_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, UPDATE ON career_change_supporter_schema.advisor_reg_req_approved To administrator_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.advisor_reg_req_approved_advisor_reg_req_approved_id_seq TO administrator_app;

CREATE TABLE career_change_supporter_schema.advisor_reg_req_rejected (
  advisor_reg_req_rejected_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address career_change_supporter_schema.email_address UNIQUE,
  last_name VARCHAR (127) NOT NULL,
  first_name VARCHAR (127) NOT NULL,
  last_name_furigana VARCHAR (127) NOT NULL,
  first_name_furigana VARCHAR (127) NOT NULL,
  /*
   * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
   * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
   */
  telephone_number VARCHAR (13) NOT NULL,
  year_of_birth SMALLINT NOT NULL,
  month_of_birth SMALLINT NOT NULL,
  day_of_birth SMALLINT NOT NULL,
  /* 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
  prefecture VARCHAR (4) NOT NULL,
  /* 市区町村の最大文字数は6文字。市区町村は都道府県と比較し、頻繁に名前が変更されるので、長さに余裕をもたせる */
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  /* 必要な文字数に応じて適宜調整 */
  reject_reason VARCHAR (1000) NOT NULL,
  rejected_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, UPDATE ON career_change_supporter_schema.advisor_reg_req_rejected To administrator_app;
GRANT USAGE ON SEQUENCE career_change_supporter_schema.advisor_reg_req_rejected_advisor_reg_req_rejected_id_seq TO administrator_app;
