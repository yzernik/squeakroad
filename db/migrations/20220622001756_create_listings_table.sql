CREATE TABLE listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price_msat UNSIGNED BIG INT NOT NULL,
    completed BOOLEAN NOT NULL,
    approved BOOLEAN NOT NULL,
    created_time_s UNSIGNED BIG INT NOT NULL
);
