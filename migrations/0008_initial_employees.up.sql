-- Add up migration script here
INSERT INTO employee (firstname, lastname, password, pw_salt, email, weekly_time, address_id) VALUES
('Luke', 'Skywalker', 'luke.skywalker@tatooine.com', 'saltA1', 'luke.skywalker@tatooine.com', '40 hours', 1),
('Padmé', 'Amidala', 'padme.amidala@naboo.com', 'saltB2', 'padme.amidala@naboo.com', '35 hours', 2),
('Han', 'Solo', 'han.solo@hoth.com', 'saltC3', 'han.solo@hoth.com', '45 hours', 3),
('Lando', 'Calrissian', 'lando.calrissian@bespin.com', 'saltD4', 'lando.calrissian@bespin.com', '30 hours', 4),
('Leia', 'Organa', 'leia.organa@coruscant.com', 'saltE5', 'leia.organa@coruscant.com', '38 hours', 5),
('Mace', 'Windu', 'mace.windu@deepcore.com', 'saltF6', 'mace.windu@deepcore.com', '42 hours', 6),
('Jar Jar', 'Binks', 'jarjar.binks@naboo.com', 'saltG7', 'jarjar.binks@naboo.com', '40 hours', 7),
('Obi-Wan', 'Kenobi', 'obiwan.kenobi@tatooine.com', 'saltH8', 'obiwan.kenobi@tatooine.com', '45 hours', 8),
('Darth', 'Vader', 'darth.vader@deathstar.com', 'saltI9', 'darth.vader@deathstar.com', '48 hours', 9),
('Yoda', '', 'yoda@dagobah.com', 'saltJ0', 'yoda@dagobah.com', '30 hours', 10);
