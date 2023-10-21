BEGIN TRANSACTION;

/* Add new fields */
DEFINE FIELD ocr ON TABLE blob TYPE string;
DEFINE FIELD nr ON TABLE song TYPE string;

/* Remove ASSERT $value != None */
DEFINE FIELD title ON TABLE song TYPE string;
DEFINE FIELD key ON TABLE song TYPE string;
DEFINE FIELD language ON TABLE song TYPE string;
DEFINE FIELD collection ON TABLE song TYPE record(collection);
DEFINE FIELD group ON TABLE song TYPE record(group);
DEFINE FIELD file_type ON TABLE blob TYPE string;
DEFINE FIELD group ON TABLE blob TYPE record(group);
DEFINE FIELD name ON TABLE group TYPE string;
DEFINE FIELD name ON TABLE user TYPE string;
DEFINE FIELD title ON TABLE collection TYPE string;
DEFINE FIELD cover ON TABLE collection TYPE string;
DEFINE FIELD group ON TABLE collection TYPE record(group);

/* Fill nr field on songs with default values */
/* REPLACE nr_sql */

UPDATE version:version SET version = 2;

COMMIT TRANSACTION;

