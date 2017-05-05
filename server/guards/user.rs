use rocket::State;
use rocket::request::{self, Request, FromRequest};
use rocket::Outcome;
use rocket::http::Status;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::ops::Deref;
use std::sync::Mutex;
use uuid::Uuid;

use models::User;

pub struct UserGuard(pub User);

impl Deref for UserGuard {
    type Target = User;

    fn deref(&self) -> &User {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserGuard {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<UserGuard, ()> {
        let uuid = match request.cookies().find("uuid") {
            Some(cookie) => Uuid::parse_str(cookie.value()).unwrap(),
            None => return Outcome::Failure((Status::new(401, "Unauthorized"), ()))
        };

        let conn = match State::<Mutex<PgConnection>>::from_request(request) {
            Outcome::Success(conn) => conn,
            _ => return Outcome::Failure((Status::new(500, "Broken DB interaction"), ()))
        };

        let conn = conn.lock().unwrap();

        use schema::user::dsl::uid;
        use schema::user;
        let user = match user::table.filter(uid.eq(uuid)).first::<User>(&*conn) {
            Ok(user) => user,
            _ => return Outcome::Failure((Status::new(401, "User doesnt exist"), ()))
        };

        return Outcome::Success(UserGuard(user));
    }
}