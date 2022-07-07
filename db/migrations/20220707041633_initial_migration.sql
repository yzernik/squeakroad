CREATE TABLE listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price_sat UNSIGNED BIG INT NOT NULL,
    fee_rate_basis_points INTEGER NOT NULL,
    reviewed BOOLEAN NOT NULL,
    submitted BOOLEAN NOT NULL,
    approved BOOLEAN NOT NULL,
    removed BOOLEAN NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL
);

CREATE UNIQUE INDEX ux_listings_public_id ON listings(public_id);

CREATE TABLE listingimages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    listing_id INTEGER NOT NULL,
    image_data BLOB NOT NULL,
    is_primary BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX ux_listingimages_public_id ON listingimages(public_id);

CREATE TABLE shippingoptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    listing_id INTEGER NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price_sat UNSIGNED BIG INT NOT NULL
);

CREATE UNIQUE INDEX ux_shippingoptions_public_id ON shippingoptions(public_id);

CREATE TABLE adminsettings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_name VARCHAR NOT NULL,
    fee_rate_basis_points INTEGER NOT NULL,
    squeaknode_address VARCHAR NOT NULL,
    squeaknode_pubkey VARCHAR NOT NULL
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    buyer_user_id INTEGER NOT NULL,
    seller_user_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    listing_id INTEGER NOT NULL,
    shipping_option_id INTEGER NOT NULL,
    shipping_instructions VARCHAR NOT NULL,
    amount_owed_sat UNSIGNED BIG INT NOT NULL,
    seller_credit_sat UNSIGNED BIG INT NOT NULL,
    paid BOOLEAN NOT NULL,
    completed BOOLEAN NOT NULL,
    acked BOOLEAN NOT NULL,
    invoice_payment_request VARCHAR NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    review_text VARCHAR NOT NULL,
    review_rating INTEGER NOT NULL,
    reviewed BOOLEAN NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    review_time_ms UNSIGNED BIG INT NOT NULL,
    payment_time_ms UNSIGNED BIG INT NOT NULL
);

CREATE UNIQUE INDEX ux_orders_public_id ON orders(public_id);

CREATE TABLE withdrawals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
    amount_sat UNSIGNED BIG INT NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    invoice_payment_request VARCHAR NOT NULL
);

CREATE UNIQUE INDEX ux_withdrawals_public_id ON withdrawals(public_id);

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

CREATE TABLE usersettings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    squeaknode_pubkey VARCHAR NOT NULL,
    squeaknode_address VARCHAR NOT NULL,
    user_id INTEGER NOT NULL
);
