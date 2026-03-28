UPDATE song SET
    search_content = (string::join(' ', array::flatten(array::flatten(
        array::flatten(data.sections.*.lines).*.parts
    ).*.languages) ?? []) ?? '');
