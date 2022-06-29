CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    listing_id INTEGER NOT NULL,
    shipping_option_id INTEGER NOT NULL,
    shipping_instructions VARCHAR NOT NULL,
    total_price_sat UNSIGNED BIG INT NOT NULL,
    seller_credit_sat UNSIGNED BIG INT NOT NULL,
    paid BOOLEAN NOT NULL,
    completed BOOLEAN NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL
);

