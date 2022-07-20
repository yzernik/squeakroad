ALTER TABLE orders
ADD COLUMN invoice_expiry_time_ms UNSIGNED BIG INT NOT NULL default 0;
