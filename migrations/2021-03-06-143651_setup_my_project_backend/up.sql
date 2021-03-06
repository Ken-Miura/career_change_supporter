CREATE SCHEMA user_data;
CREATE DOMAIN user_data.email_address AS VARCHAR (254) NOT NULL CHECK ( VALUE ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
CREATE TABLE user_data.user (
  id SERIAL PRIMARY KEY,
  mail_addr user_data.email_address,
  hashed_pass VARCHAR (64) NOT NULL
);
