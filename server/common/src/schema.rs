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
        ccs_schema.career_info (career_info_id) {
            career_info_id -> Int4,
            user_account_id -> Int4,
        }
    }

    table! {
        ccs_schema.consulting_fee (user_account_id) {
            user_account_id -> Int4,
            fee_per_hour_in_yen -> Int4,
        }
    }

    table! {
        ccs_schema.identity_info (user_account_id) {
            user_account_id -> Int4,
            last_name -> Varchar,
            first_name -> Varchar,
            last_name_furigana -> Varchar,
            first_name_furigana -> Varchar,
            sex -> Varchar,
            date_of_birth -> Date,
            prefecture -> Varchar,
            city -> Varchar,
            address_line1 -> Varchar,
            address_line2 -> Nullable<Varchar>,
            telephone_number -> Varchar,
        }
    }

    table! {
        ccs_schema.new_password (new_password_id) {
            new_password_id -> Bpchar,
            email_address -> Varchar,
            hashed_password -> Bytea,
            created_at -> Timestamptz,
        }
    }

    table! {
        ccs_schema.tenant (user_account_id) {
            user_account_id -> Int4,
            tenant_id -> Varchar,
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

    joinable!(career_info -> user_account (user_account_id));
    joinable!(consulting_fee -> user_account (user_account_id));
    joinable!(identity_info -> user_account (user_account_id));
    joinable!(tenant -> user_account (user_account_id));

    allow_tables_to_appear_in_same_query!(
        admin_account,
        career_info,
        consulting_fee,
        identity_info,
        new_password,
        tenant,
        terms_of_use,
        user_account,
        user_temp_account,
    );
}
