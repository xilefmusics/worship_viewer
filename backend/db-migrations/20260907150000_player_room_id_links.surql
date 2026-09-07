-- Store every player_room foreign ID as a real record link.
-- The participant row key is scoped by room, so its existing composite key is
-- reconstructed while converting the legacy participant ID strings.

-- Widen temporarily so existing string values can be converted in place.
DEFINE FIELD OVERWRITE host_user_id ON player_room TYPE any PERMISSIONS FULL;
DEFINE FIELD OVERWRITE host_participant_id ON player_room TYPE any PERMISSIONS FULL;
DEFINE FIELD OVERWRITE av_participant_id ON player_room TYPE any PERMISSIONS FULL;

UPDATE player_room SET
    host_user_id = IF type::is_none(host_user_id)
        THEN NONE
        ELSE type::record('user', host_user_id)
    END,
    host_participant_id = type::record(
        'player_room_participant',
        string::concat(record::id(id), ':', host_participant_id)
    ),
    av_participant_id = IF type::is_none(av_participant_id)
        THEN NONE
        ELSE type::record(
            'player_room_participant',
            string::concat(record::id(id), ':', av_participant_id)
        )
    END;

DEFINE FIELD OVERWRITE host_user_id ON player_room TYPE none | record<user> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE host_participant_id ON player_room TYPE record<player_room_participant> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE av_participant_id ON player_room TYPE none | record<player_room_participant> PERMISSIONS FULL;
