# Business logic constraints for the team invitation resource

## Static

- **BLC-TINV-001:** An invitation IS ALWAYS for one **shared** team (not a **personal** team, not the reserved catalog team).
- **BLC-TINV-002:** Creating, listing, fetching, and deleting invitations IS ALWAYS limited to team **admin** (platform **admin** has no special bypass unless the product adds it later).
- **BLC-TINV-003:** Invitations have **no expiry**, **no max uses**, and **no use counter** in the API contract.
- **BLC-TINV-004:** **DELETE** permanently removes an invitation; there IS no separate “revoked but visible” state.
- **BLC-TINV-005:** After **accept**, the invitation remains until an **admin** **DELETE**s it.
- **BLC-TINV-006:** Invitation **id** IS ALWAYS unguessable (long random identifier).

## When / then

- **BLC-TINV-007:** WHEN **POST** `/teams/{team_id}/invitations` runs THEN only a team **admin** MAY create; the team MUST be a valid shared team (invalid id THEN **400** or **404** consistent with team routes).
- **BLC-TINV-008:** WHEN **GET** list or **GET** one invitation runs THEN only team **admin** MAY; wrong team or id THEN **404** for others.
- **BLC-TINV-009:** WHEN **DELETE** an invitation runs THEN only team **admin** MAY; missing id THEN **404** vs **204** MUST stay consistent across the API.
- **BLC-TINV-010:** WHEN **accept** runs THEN the session MUST be authenticated; the current user IS added as **guest** on the team (**members** in **GET /teams/{id}**).
- **BLC-TINV-011:** WHEN **accept** runs and the user is already **content_maintainer** or **admin** on that team THEN their role MUST NOT downgrade to **guest**.
- **BLC-TINV-012:** WHEN **accept** runs and the user is already **guest** THEN duplicate **members** entries MUST NOT appear.
- **BLC-TINV-013:** WHEN **accept** succeeds and the invitation still exists THEN the same invitation id MAY be used again until an admin deletes it.
- **BLC-TINV-014:** WHEN a non-admin calls **GET** or **accept** with a wrong or foreign invitation id THEN **404**.
