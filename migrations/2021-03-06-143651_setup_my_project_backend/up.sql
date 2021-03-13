CREATE SCHEMA my_project_schema;
CREATE DOMAIN my_project_schema.email_address AS VARCHAR (254) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
CREATE TABLE my_project_schema.user (
  id SERIAL PRIMARY KEY,
  email_address my_project_schema.email_address UNIQUE,
  hashed_password BYTEA NOT NULL
);
