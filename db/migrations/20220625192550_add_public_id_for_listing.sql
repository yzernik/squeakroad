ALTER TABLE listings
ADD COLUMN public_id VARCHAR NOT NULL default '';

CREATE UNIQUE INDEX ux_listings_public_id ON listings(public_id);
