DEFINE TABLE OVERWRITE blob SCHEMAFULL;

DEFINE FIELD OVERWRITE file_type ON blob TYPE string
    ASSERT $value INSIDE ["image/png", "image/jpeg", "image/svg"];

DEFINE FIELD OVERWRITE width ON blob TYPE int
    ASSERT $value >= 0;

DEFINE FIELD OVERWRITE height ON blob TYPE int
    ASSERT $value >= 0;

DEFINE FIELD OVERWRITE ocr ON blob TYPE string
    VALUE $value ?? $before ?? "";
