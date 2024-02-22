-- Add up migration script here
CREATE TABLE IF NOT EXISTS products (
    id serial PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL,
    seller_id INT REFERENCES accounts
);