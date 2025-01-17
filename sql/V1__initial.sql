-- Revises: V0
-- Creation Date: 2025-01-17
-- Reason: Initial

CREATE TABLE IF NOT EXISTS images
(
    id         TEXT     NOT NULL PRIMARY KEY,
    image_data BYTEA    NOT NULL,
    mimetype   TEXT     NOT NULL
);