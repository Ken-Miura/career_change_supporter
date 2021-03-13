pub mod my_project_schema {
    table! {
        my_project_schema.user (id) {
            id -> Int4,
            email_address -> Varchar,
            hashed_password -> Varchar,
        }
    }
}
