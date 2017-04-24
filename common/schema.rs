use diesel::prelude::*;
use diesel::pg::PgConnection;

// TODO(loyd): use UNIX socket in production.
const DATABASE_URL: &str = "postgres://localhost/greeder";

pub fn establish_connection() -> ConnectionResult<PgConnection> {
    PgConnection::establish(DATABASE_URL)
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
