#!/bin/sh
set -eu

DB_URL="postgres://${DB_MASTER_USER}:${DB_MASTER_PASSWORD}@${DB_HOST}:${DB_PORT}/ccs_db"
sea-orm-cli migrate "${UP_OR_DOWN}" -u "${DB_URL}"
