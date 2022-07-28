CREATE TABLE useraccounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id Integer NOT NULL,
    amount_owed_sat UNSIGNED BIG INT NOT NULL,
    paid BOOLEAN NOT NULL,
    disabled boolean NOT NULL,
    invoice_payment_request VARCHAR NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    payment_time_ms UNSIGNED BIG INT NOT NULL
);

