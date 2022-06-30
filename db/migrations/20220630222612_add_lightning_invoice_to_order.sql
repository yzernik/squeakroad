ALTER TABLE orders
ADD COLUMN invoice_hash VARCHAR NOT NULL default "";

ALTER TABLE orders
ADD COLUMN invoice_payment_request VARCHAR NOT NULL default "";

