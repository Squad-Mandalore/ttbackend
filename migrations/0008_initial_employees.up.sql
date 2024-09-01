-- Add up migration script here
INSERT INTO employee (firstname, lastname, password, pw_salt, email, weekly_time, address_id) VALUES
('Luke', 'Skywalker', 'force123', 'saltA1', 'luke.skywalker@tatooine.com', '40 hours', 1),
('Padmé', 'Amidala', 'naboo789', 'saltB2', 'padme.amidala@naboo.com', '35 hours', 2),
('Han', 'Solo', 'falcon456', 'saltC3', 'han.solo@hoth.com', '45 hours', 3),
('Lando', 'Calrissian', 'cloud999', 'saltD4', 'lando.calrissian@bespin.com', '30 hours', 4),
('Leia', 'Organa', 'alderaan123', 'saltE5', 'leia.organa@coruscant.com', '38 hours', 5),
('Mace', 'Windu', 'jedi456', 'saltF6', 'mace.windu@deepcore.com', '42 hours', 6),
('Bail', 'Organa', 'senate789', 'saltG7', 'bail.organa@alderaan.com', '40 hours', 7),
('Rey', 'Skywalker', 'force1000', 'saltH8', 'rey.skywalker@batuu.com', '45 hours', 8),
('Orson', 'Krennic', 'empire123', 'saltI9', 'orson.krennic@scarif.com', '48 hours', 9),
('Ezra', 'Bridger', 'rebel789', 'saltJ0', 'ezra.bridger@lothal.com', '30 hours', 10);
