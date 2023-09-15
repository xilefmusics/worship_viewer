BEGIN TRANSACTION;

DEFINE TABLE version SCHEMAFUL;
DEFINE FIELD created ON TABLE version TYPE datetime VALUE $value OR time::now();
DEFINE FIELD version ON TABLE version TYPE int ASSERT $value > 0;

DEFINE TABLE group SCHEMAFUL;
DEFINE FIELD created ON TABLE group TYPE datetime VALUE $value OR time::now();
DEFINE FIELD name ON TABLE group TYPE string ASSERT $value != NONE;

DEFINE TABLE user SCHEMAFUL;
DEFINE FIELD created ON TABLE user TYPE datetime VALUE $value OR time::now();
DEFINE FIELD name ON TABLE user TYPE string ASSERT $value != NONE;

DEFINE TABLE member_of SCHEMAFUL;
DEFINE FIELD created ON TABLE member_of TYPE datetime VALUE $value OR time::now();

DEFINE TABLE blob SCHEMAFUL;
DEFINE FIELD created ON TABLE blob TYPE datetime VALUE $value OR time::now();
DEFINE FIELD file_type ON TABLE blob TYPE string ASSERT $value != NONE;
DEFINE FIELD width ON TABLE blob TYPE int ASSERT $value > 0;
DEFINE FIELD height ON TABLE blob TYPE int ASSERT $value > 0;
DEFINE FIELD group ON TABLE blob TYPE record(group) ASSERT $value != NONE;
DEFINE FIELD tags ON TABLE blob TYPE array;
DEFINE FIELD tags.* ON TABLE blob TYPE string;

DEFINE TABLE song SCHEMAFUL;
DEFINE FIELD created ON TABLE song TYPE datetime VALUE $value OR time::now();
DEFINE FIELD title ON TABLE song TYPE string ASSERT $value != NONE;
DEFINE FIELD key ON TABLE song TYPE string ASSERT $value != NONE;
DEFINE FIELD language ON TABLE song TYPE string ASSERT $value != NONE;
DEFINE FIELD title2 ON TABLE song TYPE string;
DEFINE FIELD key2 ON TABLE song TYPE string;
DEFINE FIELD not_a_song ON TABLE song TYPE bool;
DEFINE FIELD blobs ON TABLE song TYPE array;
DEFINE FIELD blobs.* ON TABLE song TYPE record(blob);
DEFINE FIELD collection ON TABLE song TYPE record(collection) ASSERT $value != NONE;
DEFINE FIELD group ON TABLE song TYPE record(group) ASSERT $value != NONE;
DEFINE FIELD tags ON TABLE song TYPE array;
DEFINE FIELD tags.* ON TABLE song TYPE string;

DEFINE TABLE collection SCHEMAFUL;
DEFINE FIELD created ON TABLE collection TYPE datetime VALUE $value OR time::now();
DEFINE FIELD title ON TABLE collection TYPE string ASSERT $value != NONE;
DEFINE FIELD cover ON TABLE collection TYPE string ASSERT $value != NONE;
DEFINE FIELD songs ON TABLE collection TYPE array;
DEFINE FIELD songs.* ON TABLE collection TYPE record(song);
DEFINE FIELD group ON TABLE collection TYPE record(group) ASSERT $value != NONE;
DEFINE FIELD tags ON TABLE collection TYPE array;
DEFINE FIELD tags.* ON TABLE collection TYPE string;

CREATE version:version SET version = 1;
CREATE group:admin SET name = "admin";
CREATE user:admin SET name = "admin";
RELATE user:admin->member_of->group:admin;

COMMIT TRANSACTION;
