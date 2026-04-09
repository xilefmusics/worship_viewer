DEFINE FUNCTION OVERWRITE fn::song_nonempty_strs($arr: array) -> array {
    RETURN array::filter($arr, |$x| string::len($x) > 0);
};

DEFINE FUNCTION OVERWRITE fn::song_coalesce_str_array($value: any, $before: any, $fallback: string) -> array {
    LET $arr = ($value ?? $before) ?? [];
    LET $ne = fn::song_nonempty_strs($arr);
    IF array::len($ne) > 0 {
        RETURN $ne;
    };
    RETURN [$fallback];
};

DEFINE FIELD OVERWRITE data.artists ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_coalesce_str_array($value, $before, data.artist ?? '')
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.languages ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_coalesce_str_array($value, $before, data.language ?? '')
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.titles ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_coalesce_str_array($value, $before, data.title)
    ASSERT array::len($value) > 0;

UPDATE song SET data = data;
