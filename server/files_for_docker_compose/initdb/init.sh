#!/bin/bash
set -e

# TODO: ROLEのパスワード変更＋管理方法の検討
# TODO: 本番環境（AWS上のRDS）でCREATE DATABASEを行う際、template0とtemplate1のどちらを選択すべきか要確認
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE ROLE ccs;
    CREATE ROLE user_app WITH LOGIN PASSWORD 'test1234';
    CREATE ROLE admin_app WITH LOGIN PASSWORD 'test13579';
    CREATE DATABASE ccs_db WITH OWNER = ccs TEMPLATE = template0 ENCODING = 'UTF8' LC_COLLATE = 'C' LC_CTYPE = 'C';
EOSQL