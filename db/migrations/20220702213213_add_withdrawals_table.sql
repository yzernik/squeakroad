CREATE TABLE withdrawals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id VARCHAR NOT NULL,
    user_id INTEGER NOT NULL,
    amount_sat UNSIGNED BIG INT NOT NULL,
    created_time_ms UNSIGNED BIG INT NOT NULL,
    invoice_hash VARCHAR NOT NULL,
    invoice_payment_request VARCHAR NOT NULL
);
