ALTER TABLE adminsettings
RENAME COLUMN pgp_key_id to pgp_key;

ALTER TABLE usersettings
RENAME COLUMN pgp_key_id to pgp_key;
