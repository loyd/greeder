CREATE TABLE subscription (
    user_id INTEGER NOT NULL REFERENCES "user" ON DELETE CASCADE,
    feed_id INTEGER NOT NULL REFERENCES feed ON DELETE CASCADE,

    PRIMARY KEY (user_id, feed_id)
);