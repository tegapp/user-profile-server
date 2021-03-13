-- users
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    firebase_uid TEXT NOT NULL,

    email TEXT NOT NULL,
    email_verified BOOLEAN NOT NULL,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
SELECT sqlx_manage_updated_at('users');

CREATE UNIQUE INDEX u_firebase_uid ON users (firebase_uid);

-- hosts
CREATE TABLE hosts (
    id BIGSERIAL PRIMARY KEY,

    -- server_version TEXT NOT NULL,
    -- name TEXT,
    slug TEXT NOT NULL,
    identity_public_key TEXT NOT NULL,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
SELECT sqlx_manage_updated_at('hosts');

CREATE UNIQUE INDEX h_identity_public_key on hosts (identity_public_key);

-- machines
CREATE TABLE machines (
    id BIGSERIAL PRIMARY KEY,

    host_id BIGINT NOT NULL,
    FOREIGN KEY (host_id) REFERENCES hosts (id),

    name TEXT NOT NULL,
    slug TEXT NOT NULL,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
SELECT sqlx_manage_updated_at('machines');

CREATE INDEX m_user_id on machines (host_id);
CREATE UNIQUE INDEX m_host_id_slug on machines (host_id, slug);

-- hosts_users
CREATE TABLE hosts_users (
    id BIGSERIAL PRIMARY KEY,

    user_id BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),

    host_id BIGINT NOT NULL,
    FOREIGN KEY (host_id) REFERENCES hosts (id),

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
SELECT sqlx_manage_updated_at('hosts_users');

CREATE INDEX uh_user_id on hosts_users (user_id);
CREATE UNIQUE INDEX uh_machine_id_user_id on hosts_users (user_id, host_id);
