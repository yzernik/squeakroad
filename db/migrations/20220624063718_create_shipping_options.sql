CREATE TABLE shippingoptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    listing_id INTEGER NOT NULL,
    description VARCHAR NOT NULL,
    price_sat UNSIGNED BIG INT NOT NULL
);
