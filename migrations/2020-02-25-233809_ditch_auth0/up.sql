TRUNCATE users, machines RESTART IDENTITY;

ALTER TABLE users
DROP COLUMN auth0_id;

-- ALTER TABLE users
-- DROP COLUMN name;

ALTER TABLE users
DROP COLUMN phone_number;

ALTER TABLE users
DROP COLUMN phone_number_verified;

ALTER TABLE users
ADD COLUMN firebase_uid TEXT NOT NULL;

CREATE UNIQUE INDEX firebase_uid ON users (firebase_uid);
