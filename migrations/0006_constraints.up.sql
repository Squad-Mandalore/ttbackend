ALTER TABLE employee
ADD CONSTRAINT fk_employee_address
FOREIGN KEY (address_id)
REFERENCES address (address_id)
ON DELETE CASCADE;

ALTER TABLE worktime
ADD CONSTRAINT fk_worktime_employee
FOREIGN KEY (employee_id)
REFERENCES employee (employee_id)
ON DELETE CASCADE;

ALTER TABLE worktime
ADD CONSTRAINT fk_worktime_task
FOREIGN KEY (task_id)
REFERENCES task (task_id)
ON DELETE CASCADE;
