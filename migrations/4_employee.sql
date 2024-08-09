CREATE TABLE IF NOT EXISTS employee (
    employee_id SERIAL PRIMARY KEY,
    firstname VARCHAR(64),
    lastname VARCHAR(64),
    password VARCHAR(64),
    pw_salt VARCHAR(64),
    email VARCHAR(64),
    weekly_time INTERVAL,
    address_id SERIAL
);
