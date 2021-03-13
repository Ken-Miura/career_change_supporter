use super::schema::my_project_schema::user;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
}

#[derive(Insertable)]
#[table_name = "user"]
pub struct Account<'a> {
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
}
