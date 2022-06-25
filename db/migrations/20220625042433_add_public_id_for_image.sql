ALTER TABLE listingimages
ADD COLUMN public_id VARCHAR NOT NULL default '';

CREATE UNIQUE INDEX ux_listingimages_public_id ON listingimages(public_id);
