CREATE TABLE IF NOT EXISTS worktime (
    worktime_id SERIAL PRIMARY KEY,
    employee_id SERIAL,
    task_id SERIAL,
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_time TIMESTAMPTZ,
    timeduration INTERVAL GENERATED ALWAYS AS (end_time - start_time) STORED,
    work_type WORKTIME_TYPE NOT NULL
);
