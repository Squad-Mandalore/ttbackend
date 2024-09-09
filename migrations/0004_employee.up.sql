CREATE TABLE IF NOT EXISTS employee (
    employee_id SERIAL PRIMARY KEY,
    firstname VARCHAR(64),
    lastname VARCHAR(64),
    password VARCHAR(64) NOT NULL,
    pw_salt VARCHAR(64),
    email VARCHAR(64) UNIQUE NOT NULL,
    weekly_time INTERVAL,
    address_id SERIAL NOT NULL
);
