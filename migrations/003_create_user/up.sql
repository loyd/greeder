CREATE TABLE "user" (
    id          SERIAL      PRIMARY KEY,
    uid         UUID        NOT NULL UNIQUE
);
