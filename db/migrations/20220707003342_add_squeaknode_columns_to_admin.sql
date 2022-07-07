ALTER TABLE adminsettings
ADD COLUMN squeaknode_pubkey VARCHAR NOT NULL default "";

ALTER TABLE adminsettings
ADD COLUMN squeaknode_address VARCHAR NOT NULL default "";

