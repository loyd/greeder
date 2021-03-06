CREATE TABLE feed (
    id          SERIAL      PRIMARY KEY,
    key         TEXT        NOT NULL UNIQUE,
    url         TEXT        NOT NULL,
    title       TEXT        CHECK (ltrim(title) <> ''),
    description TEXT        CHECK (ltrim(description) <> ''),
    language    CHAR(2)     CHECK (language ~ '^[a-z][a-z]$'),  -- ISO 639-1
    logo        TEXT        CHECK (ltrim(logo) <> ''),
    copyright   TEXT        CHECK (ltrim(copyright) <> ''),
    interval    INTEGER     CHECK (interval > 0),
    augmented   TIMESTAMP
);
