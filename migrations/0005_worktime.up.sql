CREATE TABLE IF NOT EXISTS worktime (
    worktime_id SERIAL PRIMARY KEY,
    employee_id SERIAL,
    task_id SERIAL,
    start_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    end_time TIMESTAMP WITH TIME ZONE,
    timeduration INTERVAL GENERATED ALWAYS AS (end_time - start_time) STORED,
    work_type WORKTIME_TYPE
);
