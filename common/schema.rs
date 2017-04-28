use std::env;
use diesel::prelude::*;
use diesel::pg::PgConnection;

// TODO(loyd): use UNIX socket in production.
const DATABASE_URL: &str = "postgres://localhost/greeder";

pub fn establish_connection() -> ConnectionResult<PgConnection> {
    let connection = PgConnection::establish(DATABASE_URL)?;

    let is_prod = env::var("G_ENV").ok().map_or(false, |env| env == "production");

    if !is_prod {
        connection.begin_test_transaction().unwrap();
    }

    Ok(connection)
}

table! {
    feed {
        id -> Integer,
        key -> Text,
        url -> Text,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        language -> Nullable<Bpchar>,
        logo -> Nullable<Text>,
        copyright -> Nullable<Text>,
        interval -> Nullable<Integer>,
        augmented -> Nullable<Timestamp>,
    }
}

table! {
    entry {
        id -> BigInt,
        key -> Text,
        feed_id -> Integer,
        url -> Nullable<Text>,
        title -> Nullable<Text>,
        author -> Nullable<Text>,
        description -> Nullable<Text>,
        content -> Nullable<Text>,
        published -> Nullable<Timestamp>,
    }
}

table! {
    subscription (user_id, feed_id) {
        user_id -> Integer,
        feed_id -> Integer,
    }
}

table! {
    user {
        id -> Integer,
        uid -> Uuid,
    }
}
