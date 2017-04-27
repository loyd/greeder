use rocket::http::{Cookies, Cookie};
use rocket::State;
use rocket::response::NamedFile;
use rocket_contrib::{Template};
use std::path::{PathBuf, Path};
use uuid::Uuid;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::sync::Mutex;
use std::io;

use models::{User, NewUser};
use schema::user;

type Connection = Mutex<PgConnection>;
type IoResult<T> = io::Result<T>;

#[derive(Serialize)]
struct Context {
    uid: String
}

#[derive(Serialize)]
struct EmptyContext;

fn get_or_create_user(conn: &State<Connection>, cookies: &Cookies) -> IoResult<String> {
    let conn = conn.lock().unwrap();
    if let Some(cookie) = cookies.find("uuid") {
        Ok(cookie.value().to_owned())
    } else {
        let new_uuid = Uuid::new_v4();
        let new_user = NewUser { uid: new_uuid.clone() };
        let insertion = diesel::insert(&new_user)
            .into(user::table)
            .get_result::<User>(&*conn);

        if let Ok(_) = insertion {
            let uuid_string = new_uuid.hyphenated().to_string();
            let cookie = Cookie::new("uuid", uuid_string.clone());
            cookies.add(cookie);
            Ok(uuid_string)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "insertion error"))
        }
    }
}

#[get("/")]
pub fn index(conn: State<Connection>, cookies: &Cookies) -> Template {
    let uid = match get_or_create_user(&conn, &cookies) {
        Ok(uid) => uid,
        _ => return Template::render("error", &EmptyContext)
    };

    Template::render("index", &Context {
        uid: uid
    })
}

#[get("/<file..>")]
fn statics(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}