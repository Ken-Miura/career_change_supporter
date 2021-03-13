#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub email_address: String,
    pub hashed_password: String,
}
