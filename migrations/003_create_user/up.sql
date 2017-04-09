-- NOTE: create "uuid-ossp" extension before use.

CREATE TABLE "user" (
    id          SERIAL      PRIMARY KEY,
    uid         UUID        NOT NULL UNIQUE DEFAULT uuid_generate_v4()
);
