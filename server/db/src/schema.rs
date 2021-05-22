pub mod career_change_supporter_schema {
    table! {
        career_change_supporter_schema.advisor_account (advisor_account_id) {
            advisor_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        career_change_supporter_schema.advisor_registration_request (advisor_registration_request_id) {
            advisor_registration_request_id -> Bpchar,
            email_address -> Varchar,
            created_at -> Timestamptz,
        }
    }

    table! {
        career_change_supporter_schema.user_account (user_account_id) {
            user_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        career_change_supporter_schema.user_temporary_account (user_temporary_account_id) {
            user_temporary_account_id -> Bpchar,
            email_address -> Varchar,
            hashed_password -> Bytea,
            created_at -> Timestamptz,
        }
    }

    allow_tables_to_appear_in_same_query!(
        advisor_account,
        advisor_registration_request,
        user_account,
        user_temporary_account,
    );
}
