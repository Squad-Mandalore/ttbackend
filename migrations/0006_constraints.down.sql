ALTER TABLE employee
DROP CONSTRAINT fk_employee_address;

ALTER TABLE worktime
DROP CONSTRAINT fk_worktime_employee;

ALTER TABLE worktime
DROP CONSTRAINT fk_worktime_task;
