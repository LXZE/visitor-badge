-- Your SQL goes here
CREATE TABLE visitors (
  id VARCHAR NOT NULL PRIMARY KEY,
  view_count INTEGER NOT NULL DEFAULT 0
);

INSERT INTO visitors
VALUES ("lxze", 0);
