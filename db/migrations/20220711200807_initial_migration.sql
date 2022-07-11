CREATE TABLE listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
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

CREATE TABLE listingimages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    listing_id INTEGER NOT NULL,
    image_data BLOB NOT NULL,
    is_primary BOOLEAN NOT NULL
);

CREATE TABLE shippingoptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    listing_id INTEGER NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price_sat UNSIGNED BIG INT NOT NULL
);

CREATE TABLE adminsettings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_name VARCHAR NOT NULL,
    fee_rate_basis_points INTEGER NOT NULL,
    pgp_key VARCHAR NOT NULL,
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
    shipped BOOLEAN NOT NULL,
    canceled_by_seller boolean NOT NULL,
    canceled_by_buyer boolean NOT NULL,
    invoice_payment_request VARCHAR NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    review_text VARCHAR NOT NULL,
    review_rating INTEGER NOT NULL,
    reviewed BOOLEAN NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    review_time_ms UNSIGNED BIG INT NOT NULL,
    payment_time_ms UNSIGNED BIG INT NOT NULL
);

CREATE TABLE withdrawals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
    amount_sat UNSIGNED BIG INT NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    invoice_payment_request VARCHAR NOT NULL
);

CREATE TABLE usersettings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    pgp_key VARCHAR NOT NULL,
    squeaknode_pubkey VARCHAR NOT NULL,
    squeaknode_address VARCHAR NOT NULL
);
