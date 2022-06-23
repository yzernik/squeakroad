CREATE TABLE listingimages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    listing_id INTEGER NOT NULL,
    image_data BLOB NOT NULL
);

