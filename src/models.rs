use super::schema::my_project_schema::tentative_user;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Insertable)]
#[table_name = "tentative_user"]
pub struct TentativeAccountInfo<'a> {
    pub query_id: &'a str,
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub registration_time: &'a chrono::DateTime<chrono::Utc>,
}
