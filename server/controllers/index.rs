use std::path::{PathBuf, Path};
use std::sync::Mutex;
use std::io::{self, Result as IoResult, ErrorKind};
use rocket::http::{Cookies, Cookie};
use rocket::State;
use rocket::response::NamedFile;
use rocket_contrib::{Template};
use uuid::Uuid;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::{User, NewUser, UserFeed, Feed, Subscription};
use schema::{user, feed};

type Connection = Mutex<PgConnection>;

#[derive(Serialize)]
struct Context {
    uid: String,
    feeds: Vec<UserFeed>
}

#[derive(Serialize)]
struct EmptyContext;

fn get_or_create_user(conn: &State<Connection>, cookies: &Cookies) -> IoResult<User> {
    let conn = conn.lock().unwrap();
    use schema::user::dsl::uid;
    if let Some(cookie) = cookies.find("uuid") {
        let uuid = Uuid::parse_str(cookie.value()).unwrap();
        let user = match user::table.filter(uid.eq(&uuid)).first::<User>(&*conn) {
            Ok(user) => user,
            Err(_) => return Err(io::Error::new(ErrorKind::Other, "Cant fetch user from DB"))
        };
        Ok(user)
    } else {
        let new_uuid = Uuid::new_v4();
        let new_user = NewUser { uid: new_uuid.clone() };
        let insertion = diesel::insert(&new_user)
            .into(user::table)
            .get_result::<User>(&*conn);
        if let Ok(user) = insertion {
            let uuid_string = new_uuid.hyphenated().to_string();
            let cookie = Cookie::new("uuid", uuid_string.clone());
            cookies.add(cookie);
            Ok(user)
        } else {
            Err(io::Error::new(ErrorKind::Other, "Cant save new user in DB"))
        }
    }
}

#[derive(Serialize)]
struct ErrorContext {
    msg: String
}

impl ErrorContext {
    fn new(e: &str) -> ErrorContext {
        ErrorContext {
            msg: e.to_string()
        }
    }
}

#[get("/")]
pub fn index(conn: State<Connection>, cookies: &Cookies) -> Template {
    let user = match get_or_create_user(&conn, &cookies) {
        Ok(uid) => uid,
        Err(e) => return Template::render("error", &ErrorContext::new("Cant fetch user"))
    };
    let conn = conn.lock().unwrap();
    use schema::subscription::dsl::user_id;
    use schema::subscription;
    use schema::feed::dsl::id as fid;

    let subscriptions = subscription::table.filter(user_id.eq(&user.id)).load::<Subscription>(&*conn);
    let subscriptions = match subscriptions {
        Ok(subs) => subs,
        Err(_) => return Template::render("error", &ErrorContext::new("Cant fetch subscriptions from DB" ))
    };

    let mut feeds = vec![];
    for sub in subscriptions {
        let feed_id = sub.feed_id;
        let part = match feed::table.filter(fid.eq(feed_id)).load::<Feed>(&*conn) {
            Ok(data) => data.into_iter().map(|feed| feed.into()),
            Err(_) => return Template::render("error", &ErrorContext::new("Cant fetch feeds from DB" ))
        };
        feeds.extend(part);
    }

    Template::render("index", &Context {
        uid: user.uid.hyphenated().to_string(),
        feeds: feeds
    })
}


#[get("/<file..>", rank=10)]
fn statics(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}