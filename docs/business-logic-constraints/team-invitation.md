# Business logic constraints for the team invitation resource

## Static

- **BLC-TINV-001:** An invitation is for one **non-public** team — either a **shared** team or a **personal** team — never the reserved catalog team.
- **BLC-TINV-002:** Creating, listing, fetching, and deleting invitations requires **team admin** on that team (on a **personal** team, the **owner** is treated as admin for this purpose). A **member** who is **not** admin receives **403**. Callers who are **not** members of the team (or use a wrong team id) receive **404** for list/get/delete, consistent with ACL hiding. Platform **admin** has no special bypass for these operations unless the product adds it later.
- **BLC-TINV-003:** Invitations have **no expiry**, **no max uses**, and **no use counter** in the API contract.
- **BLC-TINV-004:** **DELETE** permanently removes an invitation; there IS no separate “revoked but visible” state.
- **BLC-TINV-005:** After **accept**, the invitation remains until an **admin** **DELETE**s it.
- **BLC-TINV-006:** Invitation **id** IS ALWAYS unguessable (long random identifier).

## When / then

- **BLC-TINV-007:** WHEN **POST** `/teams/{team_id}/invitations` runs THEN only a team **admin** MAY create (personal team **owner** counts as admin); the team MUST exist and MUST NOT be the catalog team (invalid id THEN **404** consistent with team routes).
- **BLC-TINV-008:** WHEN **GET** list or **GET** one invitation runs THEN only team **admin** MAY; wrong team or id THEN **404** for others.
- **BLC-TINV-009:** WHEN **DELETE** an invitation runs THEN only team **admin** MAY; missing id THEN **404** vs **204** MUST stay consistent across the API.
- **BLC-TINV-010:** WHEN **accept** runs THEN the session MUST be authenticated; the current user IS added as **guest** on the team (**members** in **GET /teams/{id}**). The primary route IS **`POST /api/v1/teams/{team_id}/invitations/{invitation_id}/accept`**; **`POST /api/v1/invitations/{invitation_id}/accept`** remains supported but IS deprecated ( **`Deprecation`** / **`Sunset`** headers on responses).
- **BLC-TINV-011:** WHEN **accept** runs and the user is already **content_maintainer** or **admin** on that team THEN their role MUST NOT downgrade to **guest**.
- **BLC-TINV-012:** WHEN **accept** runs and the user is already **guest** THEN duplicate **members** entries MUST NOT appear.
- **BLC-TINV-013:** WHEN **accept** succeeds and the invitation still exists THEN the same invitation id MAY be used again until an admin deletes it.
- **BLC-TINV-014:** WHEN a non-admin calls **GET** or **accept** with a wrong or foreign invitation id THEN **404**.
