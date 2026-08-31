---
name: create-bug
description: Create and index assessed Worship Viewer bug documents from rough reports. Use when capturing or promoting a bug under docs/issues; do not use for feature ideas.
---

# Create Bug

Turn the user's report into one concise, assessed bug document and add it to the
repository's bug index.

## Workflow

1. Read `docs/issues/templates/bug-template.md` and the `## Bugs` table in
   `docs/issues/Readme.md`.
2. If no bug report was provided, ask for it and stop.
3. Derive a concise title that describes the incorrect behavior and a lowercase
   kebab-case slug. Create `docs/issues/bugs/<slug>.md`. Never overwrite an
   existing document; make the slug more specific when it already exists.
4. Follow the template structure. Replace every placeholder, give every
   assessment a 1-5 score and concise reason, and preserve the score scales.
   Keep `[← Back to issues README](../Readme.md)` as the first visible content
   after the YAML front matter. Populate the visible `## Assessment` table with
   each score as `<score> / 5` and the same reason used in metadata. Set `status`
   to `reported`, `last_reviewed` to today's date, and `owner` to `null` unless
   the user identifies one.
5. Identify affected personas and environment details from the report. Use
   `null` for unknown environment metadata. Do not invent reproduction steps,
   evidence, affected versions, causes, or workarounds; label unknowns clearly.
6. Assess severity by consequence, frequency by occurrence, reproducibility by
   how deterministically it can be triggered, and evidence by the strongest
   artifact actually available. Estimate total fix effort including regression
   tests, documentation, migrations, and rollout. Keep severity separate from
   effort and fix risk.
7. Use repository evidence when it materially improves the report. Keep the
   observed and expected behavior distinct, and put unresolved diagnostic
   details in open questions.
8. Append one row to the Bugs table with a relative document link and the area,
   primary persona, severity, frequency, reproducibility, effort, and status.
   Preserve existing rows. Escape pipe characters that would break the Markdown
   table. Preserve the `## Rough Ideas` section unless the user explicitly asks
   to promote or remove a matching rough report.
9. Validate the YAML front matter and confirm the visible assessment table and
   index row match its metadata. Report links to the created document and
   updated index. Do not create a git commit unless the user explicitly requests
   one.
