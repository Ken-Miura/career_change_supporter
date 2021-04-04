// Copyright 2021 Ken Miura
// use crate::models;
// use diesel::prelude::*;
// use diesel::PgConnection;

// pub(crate) fn find_user_by_mail_address(
//     mail_addr: &str,
//     conn: &PgConnection,
// ) -> Result<Option<models::User>, diesel::result::Error> {
//     use crate::schema::my_project_schema::user::dsl::*;
//     let result = user
//         .filter(email_address.eq(mail_addr))
//         .execute::<models::User>(conn)
//         .optional()?;
//     Ok(result)
// }
