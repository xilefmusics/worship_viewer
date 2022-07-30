pub fn replace_umlaut(string: String) -> String {
    string
        .replace("ä", "a")
        .replace("Ä", "A")
        .replace("ö", "o")
        .replace("Ö", "O")
        .replace("ü", "u")
        .replace("Ü", "U")
        .replace("ß", "s")
        .replace("’", "'")
}
