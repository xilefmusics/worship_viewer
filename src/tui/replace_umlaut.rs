pub fn replace_umlaut(string: String) -> String {
    string.replace("ä", "a").replace("ö", "o").replace("ü", "u").replace("ß", "s")
}