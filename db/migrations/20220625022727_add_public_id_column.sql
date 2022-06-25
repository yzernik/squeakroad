ALTER TABLE shippingoptions
ADD COLUMN public_id VARCHAR NOT NULL default '';

CREATE UNIQUE INDEX ux_shippingoptions_public_id ON shippingoptions(public_id);
