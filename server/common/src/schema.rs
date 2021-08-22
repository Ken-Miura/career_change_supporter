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
        ccs_schema.adv_career_approved (adv_career_approved_id) {
            adv_career_approved_id -> Int4,
            approve_adv_acc_id -> Nullable<Int4>,
            company_name -> Varchar,
            department_name -> Nullable<Varchar>,
            office -> Nullable<Varchar>,
            contract_type -> Varchar,
            profession -> Nullable<Varchar>,
            is_manager -> Bool,
            position_name -> Nullable<Varchar>,
            start_date -> Date,
            end_date -> Nullable<Date>,
            annual_income_in_man_yen -> Nullable<Int4>,
            is_new_graduate -> Bool,
            note -> Nullable<Varchar>,
            image1 -> Varchar,
            image2 -> Nullable<Varchar>,
            approved_time -> Timestamptz,
        }
    }

    table! {
        ccs_schema.adv_career_rejected (adv_career_rejected_id) {
            adv_career_rejected_id -> Int4,
            reject_adv_acc_id -> Nullable<Int4>,
            company_name -> Varchar,
            department_name -> Nullable<Varchar>,
            office -> Nullable<Varchar>,
            contract_type -> Varchar,
            profession -> Nullable<Varchar>,
            is_manager -> Bool,
            position_name -> Nullable<Varchar>,
            start_date -> Date,
            end_date -> Nullable<Date>,
            annual_income_in_man_yen -> Nullable<Int4>,
            is_new_graduate -> Bool,
            note -> Nullable<Varchar>,
            image1 -> Varchar,
            image2 -> Nullable<Varchar>,
            reject_reason -> Varchar,
            rejected_time -> Timestamptz,
        }
    }

    table! {
        ccs_schema.advisor_account (advisor_account_id) {
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
            advice_fee_in_yen -> Nullable<Int4>,
            tenant_id -> Nullable<Varchar>,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        ccs_schema.advisor_account_creation_request (advisor_acc_request_id) {
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
        ccs_schema.advisor_career (advisor_career_id) {
            advisor_career_id -> Bpchar,
            career_associated_adv_acc_id -> Nullable<Int4>,
            company_name -> Varchar,
            department_name -> Nullable<Varchar>,
            office -> Nullable<Varchar>,
            start_date -> Date,
            end_date -> Nullable<Date>,
            contract_type -> Varchar,
            profession -> Nullable<Varchar>,
            annual_income_in_yen -> Nullable<Int4>,
            is_manager -> Bool,
            position_name -> Nullable<Varchar>,
            is_new_graduate -> Bool,
            note -> Nullable<Varchar>,
        }
    }

    table! {
        ccs_schema.advisor_career_create_req (advisor_career_create_req_id) {
            advisor_career_create_req_id -> Int4,
            cre_req_adv_acc_id -> Int4,
            company_name -> Varchar,
            department_name -> Nullable<Varchar>,
            office -> Nullable<Varchar>,
            contract_type -> Varchar,
            profession -> Nullable<Varchar>,
            is_manager -> Bool,
            position_name -> Nullable<Varchar>,
            start_date -> Date,
            end_date -> Nullable<Date>,
            annual_income_in_man_yen -> Nullable<Int4>,
            is_new_graduate -> Bool,
            note -> Nullable<Varchar>,
            image1 -> Varchar,
            image2 -> Nullable<Varchar>,
            requested_time -> Timestamptz,
        }
    }

    table! {
        ccs_schema.advisor_reg_req_approved (advisor_reg_req_approved_id) {
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
        ccs_schema.advisor_reg_req_rejected (advisor_reg_req_rejected_id) {
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
        ccs_schema.advisor_registration_request (advisor_registration_request_id) {
            advisor_registration_request_id -> Bpchar,
            email_address -> Varchar,
            created_at -> Timestamptz,
        }
    }

    table! {
        ccs_schema.user_account (user_account_id) {
            user_account_id -> Int4,
            email_address -> Varchar,
            hashed_password -> Bytea,
            last_login_time -> Nullable<Timestamptz>,
        }
    }

    table! {
        ccs_schema.user_temporary_account (user_temporary_account_id) {
            user_temporary_account_id -> Bpchar,
            email_address -> Varchar,
            hashed_password -> Bytea,
            created_at -> Timestamptz,
        }
    }

    joinable!(adv_career_approved -> advisor_reg_req_approved (approve_adv_acc_id));
    joinable!(adv_career_rejected -> advisor_reg_req_approved (reject_adv_acc_id));
    joinable!(advisor_career -> advisor_account (career_associated_adv_acc_id));
    joinable!(advisor_career_create_req -> advisor_reg_req_approved (cre_req_adv_acc_id));
    joinable!(advisor_reg_req_approved -> advisor_account (associated_advisor_account_id));

    allow_tables_to_appear_in_same_query!(
        admin_account,
        adv_career_approved,
        adv_career_rejected,
        advisor_account,
        advisor_account_creation_request,
        advisor_career,
        advisor_career_create_req,
        advisor_reg_req_approved,
        advisor_reg_req_rejected,
        advisor_registration_request,
        user_account,
        user_temporary_account,
    );
}
