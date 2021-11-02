pub mod ccs_schema {
    table! {
        ccs_schema.admin_account (admin_account_id) {
            admin_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        ccs_schema.terms_of_use (user_account_id, ver) {
            user_account_id -> Int4,
            ver -> Int4,
            email_address -> Varchar,
            agreed_at -> Timestamptz,
        }
    }

    table! {
        ccs_schema.user_account (user_account_id) {
            user_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
        }
    }

    table! {
        ccs_schema.user_temp_account (user_temp_account_id) {
            user_temp_account_id -> Bpchar,
            email_address -> Varchar,
            hashed_password -> Bytea,
            created_at -> Timestamptz,
        }
    }

    allow_tables_to_appear_in_same_query!(
        admin_account,
        terms_of_use,
        user_account,
        user_temp_account,
    );
}
