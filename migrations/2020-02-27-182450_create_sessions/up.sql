CREATE TABLE "sessions" (
    "sid" text NOT NULL COLLATE "default",
    "sess" text NOT NULL,
    "expire" timestamp(6) with time zone NOT NULL
)
WITH (OIDS=FALSE);

ALTER TABLE "sessions" ADD CONSTRAINT "sessions_pkey" PRIMARY KEY ("sid") NOT DEFERRABLE INITIALLY IMMEDIATE;

CREATE INDEX "IDX_sessions_expire" ON "sessions" ("expire");
