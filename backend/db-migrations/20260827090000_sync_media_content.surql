-- Completed content is now the only persisted media lifecycle state.
DELETE media WHERE content_json IS NONE;

-- Audio/video revisions represented background replacement attempts and are
-- obsolete now that replacements complete inside the request. Failed deck
-- revisions are also obsolete; valid deck drafts are rewritten without their
-- former lifecycle state.
UPDATE media SET pending_revision_json = NONE
WHERE !string::contains(content_json, '"type":"slide_deck"');
UPDATE media SET pending_revision_json = NONE
WHERE pending_revision_json != NONE
  AND string::contains(pending_revision_json, '"processing_error"');
UPDATE media SET pending_revision_json = IF pending_revision_json = NONE {
    NONE
} ELSE {
    string::replace(
        string::replace(
            string::replace(
                string::replace(pending_revision_json, '"operation":', '"revision_id":'),
                ',"status":"processing"',
                ''
            ),
            ',"status":"ready"',
            ''
        ),
        ',"status":"failed"',
        ''
    )
};

REMOVE FIELD IF EXISTS status ON media;
REMOVE FIELD IF EXISTS declared_kind ON media;
UPDATE media UNSET status, declared_kind;
DEFINE FIELD OVERWRITE content_json ON media TYPE string ASSERT string::len($value) > 0 PERMISSIONS FULL;
