SELECT 'CREATE DATABASE query_test'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'query_test')\gexec

\c query_test

DROP TABLE IF EXISTS Users;

CREATE TABLE Users (
    username  TEXT PRIMARY KEY,
    name  TEXT NOT NULL,
    email TEXT
);

INSERT INTO Users (username, name, email) VALUES 
    ('john', 'John Doe', 'john.doe@example.com'), 
    ('jane', 'Jane Doe', NULL);
