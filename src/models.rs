#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub mail_addr: String,
    pub hashed_pass: String,
}
