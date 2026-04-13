# Business logic constraints for the user resource

## Static

- **BLC-USER-001:** **`email`** IS ALWAYS unique after normalization (trim, lowercase).
- **BLC-USER-002:** **`role`** IS ALWAYS platform **`default`** or **`admin`** (separate from team **guest** / **content_maintainer** / **admin** on teams).
- **BLC-USER-003:** Creating a user IS ALWAYS paired with creating that user’s **personal** team (**owner** 1:1).
- **BLC-USER-004:** Optional **`default_collection`** on create IS stored as provided: the API does **not** require that the collection id exist at insert time (unknown ids MAY still yield **201**).

## List pagination

- **`GET /users`** (platform admin only) supports **`page`**, **`page_size`**, and the shared rules in [list-pagination.md](./list-pagination.md).

## When / then

- **BLC-USER-005:** WHEN any authenticated user calls **GET /users/me** THEN they receive their own **User** record for the current session.
- **BLC-USER-006:** WHEN **GET /users/me** sends the session token as the raw **`Authorization`** value without a **`Bearer `** prefix THEN the server MAY still accept it (deployment-specific).
- **BLC-USER-007:** WHEN a non-admin calls **GET /users**, **GET /users/{id}**, **POST /users**, or **DELETE /users/{id}** THEN the API responds **403**.
- **BLC-USER-008:** WHEN **POST /users** uses an email that already exists THEN the API responds **409**; invalid or missing email THEN **400**.
- **BLC-USER-009:** WHEN **GET /users/{id}** runs THEN the caller MUST be platform **admin** OR otherwise be allowed by the API to read that profile; **guest** membership on someone’s **personal** team does **not** imply permission to read the **owner**’s user record (**403**).
- **BLC-USER-010:** WHEN the current user calls **GET** or **DELETE** on **`/users/me/sessions`** or **`/users/me/sessions/{id}`** THEN only sessions belonging to **me** are visible or deletable; another user’s session id THEN **404**. See [session.md](./session.md).
- **BLC-USER-011:** WHEN platform admin uses **`/users/{user_id}/sessions`** (and `{id}`) THEN they MAY list, **POST** (create), fetch, or delete that user’s sessions (session lifetime per server configuration).

## Cascading deletes

- **BLC-USER-012:** WHEN **`DELETE /users/{id}`** succeeds THEN that user’s sessions stop working; clients using only those sessions THEN get **401** on authenticated routes.
- **BLC-USER-013:** WHEN the user account IS deleted THEN their **personal** team and all blobs, songs, collections, and setlists owned by that team are removed; former **guests** or **content_maintainer** members of that personal team THEN see **404** on those resources (no reassignment—contrast **shared** team **DELETE** in [team.md](./team.md)).
- **BLC-USER-014:** WHEN **`DELETE /users/{id}`** IS repeated for the same id THEN **404**.
