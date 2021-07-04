pub mod career_change_supporter_schema {
    table! {
        career_change_supporter_schema.administrator_account (administrator_account_id) {
            administrator_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        career_change_supporter_schema.advisor_account (advisor_account_id) {
            advisor_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_name -> Varchar,
            first_name -> Varchar,
            last_name_furigana -> Varchar,
            first_name_furigana -> Varchar,
            telephone_number -> Varchar,
            date_of_birth -> Date,
            prefecture -> Varchar,
            city -> Varchar,
            address_line1 -> Varchar,
            address_line2 -> Nullable<Varchar>,
            sex -> Varchar,
            tenant_id -> Nullable<Varchar>,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        career_change_supporter_schema.advisor_account_creation_request (advisor_acc_request_id) {
            advisor_acc_request_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_name -> Varchar,
            first_name -> Varchar,
            last_name_furigana -> Varchar,
            first_name_furigana -> Varchar,
            telephone_number -> Varchar,
            date_of_birth -> Date,
            prefecture -> Varchar,
            city -> Varchar,
            address_line1 -> Varchar,
            address_line2 -> Nullable<Varchar>,
            sex -> Varchar,
            image1 -> Varchar,
            image2 -> Nullable<Varchar>,
            requested_time -> Timestamptz,
        }
    }

    table! {
        career_change_supporter_schema.advisor_career (advisor_career_id) {
            advisor_career_id -> Int4,
            company_name -> Varchar,
            department_name -> Nullable<Varchar>,
            office -> Nullable<Varchar>,
            workplace -> Nullable<Varchar>,
            start_date -> Date,
            end_date -> Nullable<Date>,
            contract_type -> Varchar,
        }
    }

    table! {
        career_change_supporter_schema.advisor_reg_req_approved (advisor_reg_req_approved_id) {
            advisor_reg_req_approved_id -> Int4,
            email_address -> Varchar,
            last_name -> Varchar,
            first_name -> Varchar,
            last_name_furigana -> Varchar,
            first_name_furigana -> Varchar,
            telephone_number -> Varchar,
            date_of_birth -> Date,
            prefecture -> Varchar,
            city -> Varchar,
            address_line1 -> Varchar,
            address_line2 -> Nullable<Varchar>,
            sex -> Varchar,
            image1 -> Varchar,
            image2 -> Nullable<Varchar>,
            associated_advisor_account_id -> Nullable<Int4>,
            approved_time -> Timestamptz,
        }
    }

    table! {
        career_change_supporter_schema.advisor_reg_req_rejected (advisor_reg_req_rejected_id) {
            advisor_reg_req_rejected_id -> Int4,
            email_address -> Varchar,
            last_name -> Varchar,
            first_name -> Varchar,
            last_name_furigana -> Varchar,
            first_name_furigana -> Varchar,
            telephone_number -> Varchar,
            date_of_birth -> Date,
            prefecture -> Varchar,
            city -> Varchar,
            address_line1 -> Varchar,
            address_line2 -> Nullable<Varchar>,
            sex -> Varchar,
            reject_reason -> Varchar,
            rejected_time -> Timestamptz,
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

    joinable!(advisor_reg_req_approved -> advisor_account (associated_advisor_account_id));

    allow_tables_to_appear_in_same_query!(
        administrator_account,
        advisor_account,
        advisor_account_creation_request,
        advisor_career,
        advisor_reg_req_approved,
        advisor_reg_req_rejected,
        advisor_registration_request,
        user_account,
        user_temporary_account,
    );
}
