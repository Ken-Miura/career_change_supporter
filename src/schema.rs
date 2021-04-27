pub mod my_project_schema {
    table! {
        my_project_schema.user_account (user_account_id) {
            user_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        my_project_schema.user_temporary_account (user_temporary_account_id) {
            user_temporary_account_id -> Bpchar,
            email_address -> Varchar,
            hashed_password -> Bytea,
            created_at -> Timestamptz,
        }
    }

    allow_tables_to_appear_in_same_query!(
        user_account,
        user_temporary_account,
    );
}
