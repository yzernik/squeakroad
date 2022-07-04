CREATE TABLE ordermessages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    order_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    recipient_id INTEGER NOT NULL,
    text VARCHAR NOT NULL,
    viewed BOOLEAN NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL
);
