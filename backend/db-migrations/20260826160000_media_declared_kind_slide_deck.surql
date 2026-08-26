DEFINE FIELD OVERWRITE declared_kind ON media TYPE none | string ASSERT $value = NONE OR $value IN ['video', 'audio', 'slide_deck'] PERMISSIONS FULL;
