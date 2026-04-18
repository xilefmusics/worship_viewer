# Create vs update request bodies

## Rule

- Use **`Create*`** for **POST** (new resource, server assigns id where applicable).
- Use **`Update*`** for **PUT** when the operation is a full replacement of client-writable fields on an **existing** id, or when documenting upsert semantics separately (see songs).
- Use **`Patch*`** for **PATCH** (partial update, absent fields unchanged).

If POST and PUT accept the **same** fields and differ only by id placement, `Update*` may mirror `Create*` (duplicate struct) so OpenAPI and clients distinguish create vs replace intent.

## Upsert

- **Songs:** `PUT /api/v1/songs/{id}` may return **201 Created** with `Location` when the id did not exist (upsert). Other resources return **404** for unknown ids on PUT unless documented otherwise.

## Team

`CreateTeam`, `UpdateTeam`, and `PatchTeam` already follow this split.
