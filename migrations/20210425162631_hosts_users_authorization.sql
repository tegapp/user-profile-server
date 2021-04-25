ALTER TABLE hosts_users RENAME TO host_users;

ALTER TABLE host_users
  ADD COLUMN authorized_by_user BOOLEAN DEFAULT FALSE,
  ADD COLUMN authorized_by_host BOOLEAN DEFAULT FALSE;

CREATE UNIQUE INDEX uh_machine_id_user_id_auth on host_users (
  user_id,
  host_id,
  authorized_by_user,
  authorized_by_host
);

UPDATE host_users SET
  authorized_by_user = TRUE,
  authorized_by_host = TRUE;
