ALTER TABLE orders
ADD COLUMN public_id VARCHAR NOT NULL default '';

CREATE UNIQUE INDEX ux_orders_public_id ON orders(public_id);
