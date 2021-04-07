pub mod my_project_schema {
    table! {
        my_project_schema.tentative_user (id) {
            id -> Int4,
            query_id -> Bpchar,
            email_address -> Varchar,
            hashed_password -> Bytea,
            registration_time -> Timestamptz,
        }
    }

    table! {
        my_project_schema.user (id) {
            id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    allow_tables_to_appear_in_same_query!(
        tentative_user,
        user,
    );
}
