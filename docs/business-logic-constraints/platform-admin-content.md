# Platform admin and team library content

Cross-cutting rule for **library** resources (songs, collections, setlists, blobs) and team visibility.

## Rule

- **BLC-ADMIN-001:** A platform **`admin`** user **MAY** see additional **non-public** teams and their content in **read** paths (listing and **GET**) where the resolver grants expanded read scope (`content_read_team_things`).
- **BLC-ADMIN-002:** Platform **`admin`** does **not** receive **library edit** (mutate) rights on a team’s library **solely** because **`role = admin`** on the user. **PUT**, **PATCH**, **DELETE**, and moves require the same team **library edit** membership as non-admins (`content_write_team_things`).

Resource-specific wording appears in **BLC-SONG-002**, **BLC-COLL-002**, **BLC-SETL-002**, **BLC-BLOB-002**, and related **move** rules; this document is the single cross-reference for the invariant.
