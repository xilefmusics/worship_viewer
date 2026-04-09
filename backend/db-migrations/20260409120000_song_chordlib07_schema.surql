DEFINE FIELD OVERWRITE data.titles ON song TYPE option<array<string>>;

DEFINE FIELD OVERWRITE data.artists ON song TYPE option<array<string>>;

DEFINE FIELD OVERWRITE data.languages ON song TYPE option<array<string>>;

DEFINE FIELD OVERWRITE data.tags ON song TYPE option<object>;

DEFINE FIELD OVERWRITE data.tags.* ON song TYPE string;

DEFINE FIELD OVERWRITE data.sections.*.repeat_count ON song TYPE int
    VALUE $value ?? $before ?? 1
    ASSERT $value >= 1;

DEFINE FUNCTION OVERWRITE fn::chordlib07_time_ok($data: object) -> bool {
    LET $t = $data.time;
    IF $t IS NONE OR $t IS NULL {
        RETURN true;
    };
    IF type::is::array($t) AND array::len($t) = 2 {
        LET $d = $t[1];
        RETURN $d = 4 OR $d = 8;
    };
    RETURN false;
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_denom($data: object) -> int {
    LET $t = $data.time;
    IF $t IS NONE OR $t IS NULL {
        RETURN 4;
    };
    IF type::is::array($t) AND array::len($t) = 2 {
        RETURN <int>$t[1];
    };
    RETURN 4;
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_scale_duration($dur: int, $den: int) -> int {
    IF $den = 4 {
        RETURN ($dur * 125) / 3;
    };
    RETURN ($dur * 250) / 3;
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_chord($chord: object, $den: int) -> object {
    LET $d = $chord.duration;
    IF $d IS NONE OR $d IS NULL {
        RETURN $chord;
    };
    LET $nd = fn::chordlib07_scale_duration(<int>$d, $den);
    LET $b = $chord.base;
    IF $b IS NONE OR $b IS NULL {
        RETURN {
            main: $chord.main,
            kind: $chord.kind,
            var: $chord.var ?? "",
            duration: $nd,
            optional: $chord.optional ?? false
        };
    };
    RETURN {
        main: $chord.main,
        base: $b,
        kind: $chord.kind,
        var: $chord.var ?? "",
        duration: $nd,
        optional: $chord.optional ?? false
    };
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_part($part: object, $den: int) -> object {
    LET $c = $part.chord;
    IF $c IS NONE OR $c IS NULL {
        RETURN $part;
    };
    RETURN {
        chord: fn::chordlib07_migrate_chord($c, $den),
        languages: $part.languages,
        comment: $part.comment ?? false
    };
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_parts_rec($parts: array, $den: int, $i: int, $acc: array) -> array {
    IF $i >= array::len($parts) {
        RETURN $acc;
    };
    LET $p = fn::chordlib07_migrate_part($parts[$i], $den);
    RETURN fn::chordlib07_migrate_parts_rec($parts, $den, $i + 1, array::append($acc, $p));
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_line($line: object, $den: int) -> object {
    LET $parts = $line.parts ?? [];
    LET $np = fn::chordlib07_migrate_parts_rec($parts, $den, 0, []);
    RETURN { parts: $np };
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_lines_rec($lines: array, $den: int, $i: int, $acc: array) -> array {
    IF $i >= array::len($lines) {
        RETURN $acc;
    };
    LET $ln = fn::chordlib07_migrate_line($lines[$i], $den);
    RETURN fn::chordlib07_migrate_lines_rec($lines, $den, $i + 1, array::append($acc, $ln));
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_section($sec: object, $den: int) -> object {
    LET $lines = $sec.lines ?? [];
    LET $nl = fn::chordlib07_migrate_lines_rec($lines, $den, 0, []);
    LET $rc2 = math::max([1, <int>($sec.repeat_count ?? 1)]);
    RETURN {
        title: $sec.title,
        lines: $nl,
        repeat_count: $rc2
    };
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_sections_rec($sections: array, $den: int, $i: int, $acc: array) -> array {
    IF $i >= array::len($sections) {
        RETURN $acc;
    };
    LET $s = fn::chordlib07_migrate_section($sections[$i], $den);
    RETURN fn::chordlib07_migrate_sections_rec($sections, $den, $i + 1, array::append($acc, $s));
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_migrate_data($data: object) -> object {
    LET $den = fn::chordlib07_denom($data);
    LET $secs = $data.sections ?? [];
    LET $ns = fn::chordlib07_migrate_sections_rec($secs, $den, 0, []);
    RETURN {
        title: $data.title,
        titles: $data.titles,
        subtitle: $data.subtitle,
        copyright: $data.copyright,
        key: $data.key,
        artist: $data.artist,
        artists: $data.artists,
        language: $data.language,
        languages: $data.languages,
        tempo: $data.tempo,
        time: $data.time,
        tags: $data.tags,
        sections: $ns
    };
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_line_texts($line: object) -> array {
    LET $parts = $line.parts ?? [];
    RETURN array::flatten($parts.map(|$pt| $pt.languages ?? []));
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_section_texts($sec: object) -> array {
    LET $lines = $sec.lines ?? [];
    RETURN array::flatten($lines.map(|$ln| fn::chordlib07_line_texts($ln)));
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_nonempty_join($texts: array) -> string {
    LET $f = array::filter($texts, |$x| string::len($x) > 0);
    RETURN array::join($f, " ");
};

DEFINE FUNCTION OVERWRITE fn::chordlib07_search_content($data: object) -> string {
    LET $secs = $data.sections ?? [];
    LET $all = array::flatten($secs.map(|$s| fn::chordlib07_section_texts($s)));
    RETURN fn::chordlib07_nonempty_join($all);
};

LET $bad = SELECT id FROM song WHERE !fn::chordlib07_time_ok(data);

IF array::len($bad) > 0 {
    THROW "chordlib 0.7 migration: unsupported time signature denominator (expected 4 or 8)";
};

UPDATE song SET
    data = fn::chordlib07_migrate_data(data),
    search_content = fn::chordlib07_search_content(fn::chordlib07_migrate_data(data));

REMOVE FIELD IF EXISTS chordlib07_migrated ON TABLE song;

REMOVE FUNCTION IF EXISTS fn::chordlib07_time_ok;
REMOVE FUNCTION IF EXISTS fn::chordlib07_denom;
REMOVE FUNCTION IF EXISTS fn::chordlib07_scale_duration;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_chord;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_part;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_parts_rec;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_line;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_lines_rec;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_section;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_sections_rec;
REMOVE FUNCTION IF EXISTS fn::chordlib07_migrate_data;
REMOVE FUNCTION IF EXISTS fn::chordlib07_line_texts;
REMOVE FUNCTION IF EXISTS fn::chordlib07_section_texts;
REMOVE FUNCTION IF EXISTS fn::chordlib07_nonempty_join;
REMOVE FUNCTION IF EXISTS fn::chordlib07_search_content;
