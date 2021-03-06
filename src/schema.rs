pub mod user_data {
    table! {
        user_data.user (id) {
            id -> Int4,
            mail_addr -> Varchar,
            hashed_pass -> Varchar,
        }
    }
}
