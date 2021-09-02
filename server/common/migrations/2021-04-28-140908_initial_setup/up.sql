/* TODO: パスワード変更＋管理方法の検討 */
CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';
CREATE ROLE advisor_app WITH LOGIN PASSWORD 'test5678';
CREATE ROLE admin_app WITH LOGIN PASSWORD 'test13579';
CREATE ROLE admin_account_app WITH LOGIN PASSWORD 'test24680';

/* ccs = Career Change Supporter */
CREATE SCHEMA ccs_schema;
GRANT USAGE ON SCHEMA ccs_schema TO user_app;
GRANT USAGE ON SCHEMA ccs_schema TO advisor_app;
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
  last_login_time TIMESTAMP WITH TIME ZONE
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

/* data structure for advisor */
CREATE TABLE ccs_schema.advisor_registration_request (
  advisor_registration_request_id ccs_schema.uuid_simple_form PRIMARY KEY,
  /* 一度登録依頼を出した後、それを忘れてしまいもう一度登録依頼したいケースを考え、UNIQUEにしない。advisor_registration_request_idがPRIMARY KEYなので一意に検索は可能 */
  email_address ccs_schema.email_address,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT, DELETE ON ccs_schema.advisor_registration_request To advisor_app;

/* TODO: 必要情報のアップロード処理が追加され次第、データを追加して更新する */
CREATE TABLE ccs_schema.advisor_account (
  advisor_account_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address ccs_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL,
  last_name VARCHAR (127) NOT NULL,
  first_name VARCHAR (127) NOT NULL,
  last_name_furigana VARCHAR (127) NOT NULL,
  first_name_furigana VARCHAR (127) NOT NULL,
  telephone_number VARCHAR (13) NOT NULL,
  date_of_birth DATE NOT NULL,
  prefecture VARCHAR (4) NOT NULL,
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  sex ccs_schema.sex,
  advice_fee_in_yen INTEGER,
  tenant_id ccs_schema.tenant_id UNIQUE,
  last_login_time TIMESTAMP WITH TIME ZONE
);
GRANT SELECT, UPDATE, DELETE ON ccs_schema.advisor_account To advisor_app;
/* TODO: 単なる読み取りだけの場合はSEQUENCEの権限を与える必要はないはずだが、確認する */
/*GRANT USAGE ON SEQUENCE ccs_schema.advisor_account_advisor_account_id_seq TO advisor_app;*/

GRANT SELECT, INSERT, UPDATE ON ccs_schema.advisor_account To admin_app;
GRANT USAGE ON SEQUENCE ccs_schema.advisor_account_advisor_account_id_seq TO admin_app;

GRANT SELECT (advisor_account_id, sex, advice_fee_in_yen) ON ccs_schema.advisor_account To user_app;

CREATE TABLE ccs_schema.advisor_account_creation_request (
  advisor_acc_request_id SERIAL PRIMARY KEY,
  /* NOTE: email_addressがUNIQUEであることに依存するコードとなっているため、UNIQUEを外さない */
  email_address ccs_schema.email_address UNIQUE,
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
  date_of_birth DATE NOT NULL,
  /* 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
  prefecture VARCHAR (4) NOT NULL,
  /* 市区町村の最大文字数は6文字。市区町村は都道府県と比較し、頻繁に名前が変更されるので、長さに余裕をもたせる */
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  sex ccs_schema.sex,
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
  requested_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT INSERT ON ccs_schema.advisor_account_creation_request To advisor_app;
GRANT SELECT (email_address) ON ccs_schema.advisor_account_creation_request To advisor_app;
GRANT USAGE ON SEQUENCE ccs_schema.advisor_account_creation_request_advisor_acc_request_id_seq TO advisor_app;
GRANT SELECT, DELETE ON ccs_schema.advisor_account_creation_request To admin_app;

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

CREATE TABLE ccs_schema.advisor_reg_req_approved (
  advisor_reg_req_approved_id SERIAL PRIMARY KEY,
  /* NOTE: UNIQUEでない可能性があるのでUNIQUEはつけない */
  email_address ccs_schema.email_address,
  last_name VARCHAR (127) NOT NULL,
  first_name VARCHAR (127) NOT NULL,
  last_name_furigana VARCHAR (127) NOT NULL,
  first_name_furigana VARCHAR (127) NOT NULL,
  /*
   * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
   * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
   */
  telephone_number VARCHAR (13) NOT NULL,
  date_of_birth DATE NOT NULL,
  /* 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
  prefecture VARCHAR (4) NOT NULL,
  /* 市区町村の最大文字数は6文字。市区町村は都道府県と比較し、頻繁に名前が変更されるので、長さに余裕をもたせる */
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  sex ccs_schema.sex,
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
  /* 
   * SERIALはINTEGERで格納可能 https://www.postgresql.org/docs/9.1/datatype-numeric.html
   * TODO: このような使い方 (プライマリキーのシリアルをNULLABLEの値に格納する) がアンチパターンでないか調べる
   */
  associated_advisor_account_id INTEGER REFERENCES ccs_schema.advisor_account(advisor_account_id) ON DELETE SET NULL ON UPDATE CASCADE,
  approved_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT ON ccs_schema.advisor_reg_req_approved To admin_app;
GRANT USAGE ON SEQUENCE ccs_schema.advisor_reg_req_approved_advisor_reg_req_approved_id_seq TO admin_app;
GRANT SELECT (advisor_reg_req_approved_id, associated_advisor_account_id, approved_time) ON ccs_schema.advisor_reg_req_approved To advisor_app;

CREATE TABLE ccs_schema.advisor_reg_req_rejected (
  advisor_reg_req_rejected_id SERIAL PRIMARY KEY,
  /* NOTE: UNIQUEでない可能性があるのでUNIQUEはつけない */
  email_address ccs_schema.email_address,
  last_name VARCHAR (127) NOT NULL,
  first_name VARCHAR (127) NOT NULL,
  last_name_furigana VARCHAR (127) NOT NULL,
  first_name_furigana VARCHAR (127) NOT NULL,
  /*
   * 電話番号の最大桁数は15桁、国内向けのみのサービスを考えているので最大13桁とする。 
   * 参考: https://www.accumu.jp/vol22-23/%E3%82%84%E3%81%95%E3%81%97%E3%81%9D%E3%81%86%E3%81%AB%E8%A6%8B%E3%81%88%E3%82%8B%E9%9B%BB%E8%A9%B1%E7%95%AA%E5%8F%B7%E3%81%AE%E9%9B%A3%E3%81%97%E3%81%95%20%E7%B7%8F%E5%8B%99%E5%A4%A7%E8%87%A3%E8%B3%9E%E3%82%92%E5%8F%97%E8%B3%9E%E3%81%97%E3%81%A6.html#:~:text=%E6%97%A5%E6%9C%AC%E3%81%AE%E5%A0%B4%E5%90%88%EF%BC%8C%E5%9B%BD%E7%95%AA%E5%8F%B7,%E3%81%AF%E9%99%A4%E3%81%84%E3%81%A6%E6%95%B0%E3%81%88%E3%81%BE%E3%81%99%E3%80%82
   */
  telephone_number VARCHAR (13) NOT NULL,
  date_of_birth DATE NOT NULL,
  /* 都道府県の最大文字数は4文字（神奈川県、鹿児島県、和歌山県） */
  prefecture VARCHAR (4) NOT NULL,
  /* 市区町村の最大文字数は6文字。市区町村は都道府県と比較し、頻繁に名前が変更されるので、長さに余裕をもたせる */
  city VARCHAR (32) NOT NULL,
  address_line1 VARCHAR (127) NOT NULL,
  address_line2 VARCHAR (127),
  sex ccs_schema.sex,
  /* 必要な文字数に応じて適宜調整 */
  reject_reason VARCHAR (1000) NOT NULL,
  rejected_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT ON ccs_schema.advisor_reg_req_rejected To admin_app;
GRANT USAGE ON SEQUENCE ccs_schema.advisor_reg_req_rejected_advisor_reg_req_rejected_id_seq TO admin_app;

CREATE TABLE ccs_schema.advisor_career (
  advisor_career_id ccs_schema.uuid_simple_form PRIMARY KEY,
  career_associated_adv_acc_id INTEGER REFERENCES ccs_schema.advisor_account(advisor_account_id) ON DELETE CASCADE ON UPDATE CASCADE,
  company_name VARCHAR (1000) NOT NULL, /* 会社名 */
  department_name VARCHAR (1000), /* 事業部・部門 */
  office VARCHAR (1000), /* 事業所 */
  start_date DATE NOT NULL, /* start_dateとend_dateから在籍期間を表示するようにする */
  end_date DATE,
  contract_type VARCHAR (100) NOT NULL, /* 正社員か契約社員かその他か */
  profession VARCHAR (100), /* 職種 */
  annual_income_in_yen INTEGER, /* 年収 */
  is_manager BOOLEAN NOT NULL, /* 管理職かどうか */
  position_name VARCHAR (100), /* 役職名 */
  is_new_graduate BOOLEAN NOT NULL, /* 入社区分（新卒、中途） */
  note VARCHAR (2000) /* その他備考 (相談可能な内容、相談不可な内容) */
);

/* TODO: advisor自身で更新可能なカラムは個別にUPDATE権限を入れる */
GRANT SELECT ON ccs_schema.advisor_career To advisor_app;

GRANT SELECT, INSERT, UPDATE ON ccs_schema.advisor_career To admin_app;

GRANT SELECT ON ccs_schema.advisor_career To user_app;

CREATE TABLE ccs_schema.advisor_career_create_req (
  advisor_career_create_req_id SERIAL PRIMARY KEY,
  /* advisor_reg_req_approved_idの変更や削除は基本的に許可しない予定なのでデフォルト（エラー）動作 */
  cre_req_adv_acc_id INTEGER NOT NULL REFERENCES ccs_schema.advisor_reg_req_approved(advisor_reg_req_approved_id),
  company_name VARCHAR (1000) NOT NULL, /* 会社名 */
  department_name VARCHAR (1000), /* 事業部・部門 */
  office VARCHAR (1000), /* 事業所 */
  contract_type VARCHAR (100) NOT NULL, /* 正社員か契約社員かその他か */
  profession VARCHAR (100), /* 職種 */
  is_manager BOOLEAN NOT NULL, /* 管理職かどうか */
  position_name VARCHAR (100), /* 役職名 */
  /* 以下は名刺からわからないのでチェックしない */
  start_date DATE NOT NULL, /* start_dateとend_dateから在籍期間を表示するようにする */
  end_date DATE,
  annual_income_in_man_yen INTEGER, /* 年収 */
  is_new_graduate BOOLEAN NOT NULL, /* 入社区分（新卒、中途） */
  note VARCHAR (2000), /* その他備考 (相談可能な内容、相談不可な内容) */
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
  requested_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT INSERT ON ccs_schema.advisor_career_create_req To advisor_app;
GRANT USAGE ON SEQUENCE ccs_schema.advisor_career_create_req_advisor_career_create_req_id_seq TO advisor_app;

GRANT SELECT, DELETE ON ccs_schema.advisor_career_create_req To admin_app;

CREATE TABLE ccs_schema.adv_career_approved (
  adv_career_approved_id SERIAL PRIMARY KEY,
  /* advisor_reg_req_approved_idの変更や削除は基本的に許可しない予定なのでデフォルト（エラー）動作 */
  approve_adv_acc_id INTEGER REFERENCES ccs_schema.advisor_reg_req_approved(advisor_reg_req_approved_id),
  company_name VARCHAR (1000) NOT NULL, /* 会社名 */
  department_name VARCHAR (1000), /* 事業部・部門 */
  office VARCHAR (1000), /* 事業所 */
  contract_type VARCHAR (100) NOT NULL, /* 正社員か契約社員かその他か */
  profession VARCHAR (100), /* 職種 */
  is_manager BOOLEAN NOT NULL, /* 管理職かどうか */
  position_name VARCHAR (100), /* 役職名 */
  /* TODO: 以下の項目はどうすべきか検討 */
  start_date DATE NOT NULL, /* start_dateとend_dateから在籍期間を表示するようにする */
  end_date DATE,
  annual_income_in_man_yen INTEGER, /* 年収 */
  is_new_graduate BOOLEAN NOT NULL, /* 入社区分（新卒、中途） */
  note VARCHAR (2000), /* その他備考 (相談可能な内容、相談不可な内容) */
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
  approved_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT ON ccs_schema.adv_career_approved To admin_app;
GRANT USAGE ON SEQUENCE ccs_schema.adv_career_approved_adv_career_approved_id_seq TO admin_app;

CREATE TABLE ccs_schema.adv_career_rejected (
  adv_career_rejected_id SERIAL PRIMARY KEY,
  /* advisor_reg_req_approved_idの変更や削除は基本的に許可しない予定なのでデフォルト（エラー）動作 */
  reject_adv_acc_id INTEGER REFERENCES ccs_schema.advisor_reg_req_approved(advisor_reg_req_approved_id),
  company_name VARCHAR (1000) NOT NULL, /* 会社名 */
  department_name VARCHAR (1000), /* 事業部・部門 */
  office VARCHAR (1000), /* 事業所 */
  contract_type VARCHAR (100) NOT NULL, /* 正社員か契約社員かその他か */
  profession VARCHAR (100), /* 職種 */
  is_manager BOOLEAN NOT NULL, /* 管理職かどうか */
  position_name VARCHAR (100), /* 役職名 */
  /* TODO: 以下の項目はどうすべきか検討 */
  start_date DATE NOT NULL, /* start_dateとend_dateから在籍期間を表示するようにする */
  end_date DATE,
  annual_income_in_man_yen INTEGER, /* 年収 */
  is_new_graduate BOOLEAN NOT NULL, /* 入社区分（新卒、中途） */
  note VARCHAR (2000), /* その他備考 (相談可能な内容、相談不可な内容) */
  /* TODO: 最大文字数の検討 */
  image1 VARCHAR (64) NOT NULL,
  image2 VARCHAR (64),
    /* 必要な文字数に応じて適宜調整 */
  reject_reason VARCHAR (1000) NOT NULL,
  rejected_time TIMESTAMP WITH TIME ZONE NOT NULL
);
GRANT SELECT, INSERT ON ccs_schema.adv_career_rejected To admin_app;
GRANT USAGE ON SEQUENCE ccs_schema.adv_career_rejected_adv_career_rejected_id_seq TO admin_app;
