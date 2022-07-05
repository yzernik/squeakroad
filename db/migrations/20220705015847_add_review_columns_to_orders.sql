ALTER TABLE orders
ADD COLUMN reviewed BOOLEAN NOT NULL default 0;

ALTER TABLE orders
ADD COLUMN review_rating INTEGER NOT NULL default 0;

ALTER TABLE orders
ADD COLUMN review_text VARCHAR NOT NULL default "";

ALTER TABLE orders
ADD COLUMN review_time_ms UNSIGNED BIG INT NOT NULL default "";
