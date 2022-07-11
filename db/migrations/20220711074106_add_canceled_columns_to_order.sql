ALTER TABLE orders
ADD COLUMN canceled_by_seller boolean NOT NULL default 0;

ALTER TABLE orders
ADD COLUMN canceled_by_buyer boolean NOT NULL default 0;
