-- Add up migration script here
CREATE TABLE IF NOT EXISTS accounts (
    id serial PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(255) NOT NULL,
    created_on TIMESTAMP NOT NULL DEFAULT NOW()
);