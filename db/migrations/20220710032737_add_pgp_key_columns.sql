ALTER TABLE adminsettings
ADD COLUMN pgp_key_id VARCHAR NOT NULL default '';

ALTER TABLE usersettings
ADD COLUMN pgp_key_id VARCHAR NOT NULL default '';
