-- Accept registered SVG MIME type; keep legacy image/svg for existing rows.
DEFINE FIELD OVERWRITE file_type ON blob TYPE string
    ASSERT $value INSIDE ["image/png", "image/jpeg", "image/svg", "image/svg+xml"];
