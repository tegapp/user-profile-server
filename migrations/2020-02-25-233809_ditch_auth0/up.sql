TRUNCATE users, machines RESTART IDENTITY;

ALTER TABLE users
DROP COLUMN auth0_id;

ALTER TABLE users
DROP COLUMN name;

-- ALTER TABLE users
-- DROP COLUMN phone_number;

-- ALTER TABLE users
-- DROP COLUMN phone_number_verified;

ALTER TABLE users
ADD COLUMN username TEXT NOT NULL;

ALTER TABLE users
ADD COLUMN hashed_password TEXT;

CREATE UNIQUE INDEX uniq_username ON users (username);
