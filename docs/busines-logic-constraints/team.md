# Business logic constraints for the team resource

## General

- **Kinds:** *Personal* team = 1:1 with a user; that user is **owner**. *Shared* team = collaboration team; **no** `owner` in the personal sense—creator is **admin** in `members`; admins are peers.
- **Personal owner:** Not listed in `members`. Effective auth treats the owner as **≥ admin** on the team and on team-owned resources.
- **Liveness:** The team always has **at least one** of: a personal **owner**, or an **admin** in `members` (operational rules—e.g. leave-team blocking—preserve this where the product requires it).
- **Readable team ids (auth):** (1) teams where the user is in `members`, (2) the personal team whose **owner** is the user, **or** (3) any team when the authenticated user is a **platform admin** (`User.role` = admin)—read-only for list/GET; membership rules for writes are unchanged. The **system/public** team row (e.g. `team:public`) may exist in the database for **internal** catalog/auth logic only; it is **not** exposed through the team REST API (treat like unknown id: 404), including for platform admins.

## Create

- Users may create arbitrarily many shared teams (**no** hard quota). The creating user is always an **admin** member. **POST** may include an optional `members` list of additional `{ user: { id }, role }` entries (creator is ignored if duplicated there; creator stays **admin**).
- Only **shared** teams use this flow; **personal** teams are created when the user is created.
- Team **display names** need not be unique (globally or per user).
- **Public team:** Users may read public catalog resources **without** being members of the public team.

## Read

- Anyone in `members` may read the team **regardless** of role (Guest+), per the role matrix.
- A user’s readable teams = teams where they are in `members` **∪** the personal team where they are **`owner`** (not in `members` there) **∪** (if platform admin) **all** non–`team:public` teams.
- **GET** JSON: `owner` and each `members[].user` are `{ id, email }` (no full user resource).

## Update

- Team metadata and membership: **PUT** carries `name` and optionally `members`. When `members` is present, it **replaces** the full member list; each entry is `{ user: { id }, role }` (shared teams must still have **≥ one admin**; personal-team **owner** must not appear in `members`).
- Principals with **effective role ≥ admin**—the personal-team **owner** or an **admin** member—may **PUT** any combination of `name` and optional full `members` replacement (subject to the rules above).
- **Content maintainer** and **guest** may **only** **PUT** a **self-leave**: `name` unchanged and `members` exactly the current list **minus themselves**. Any other change (rename, add/remove other members) is rejected.
- Personal team **`owner`** cannot be changed (stays 1:1 with that user).
- **Sole admin** on a shared team cannot end up with **no** admin in `members` (e.g. clearing `members` or removing the last admin): **409** until another admin exists or the team is deleted.

## Delete

- Only **shared** teams may be deleted; actor must have **effective role ≥ admin**. **Personal** teams cannot be deleted.
- Deleting a shared team **moves all of that team’s resources** to the **personal team of the admin who performed the delete**.
