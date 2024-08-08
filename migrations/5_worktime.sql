CREATE TABLE IF NOT EXISTS worktime (
    worktime_id SERIAL PRIMARY KEY,
    employee_id SERIAL,
    task_id SERIAL,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    timeduration INTERVAL,
    type WORKTIME_TYPE
);
