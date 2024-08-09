CREATE TABLE IF NOT EXISTS address (
    address_id SERIAL PRIMARY KEY,
    city VARCHAR(64),
    plz VARCHAR(5),
    street VARCHAR(64),
    house_no VARCHAR(64)
);
