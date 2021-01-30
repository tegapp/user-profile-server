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

-- machines
CREATE TABLE machines (
    id BIGSERIAL PRIMARY KEY,

    user_id BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),

    public_key TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
SELECT sqlx_manage_updated_at('machines');

-- users_machines
CREATE TABLE users_machines (
    id BIGSERIAL PRIMARY KEY,

    user_id BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),

    machine_id BIGINT NOT NULL,
    FOREIGN KEY (machine_id) REFERENCES users (id),

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
SELECT sqlx_manage_updated_at('users_machines');

CREATE UNIQUE INDEX um_user_id on users_machines (user_id);
CREATE UNIQUE INDEX um_machine_id_user_id on users_machines (user_id, machine_id);
