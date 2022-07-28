ALTER TABLE listings
ADD COLUMN deactivated_by_seller boolean NOT NULL default 0;

ALTER TABLE listings
ADD COLUMN deactivated_by_admin boolean NOT NULL default 0;
