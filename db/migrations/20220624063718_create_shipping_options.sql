CREATE TABLE shippingoptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    listing_id INTEGER NOT NULL,
    description VARCHAR NOT NULL,
    price_msat UNSIGNED BIG INT NOT NULL
);
