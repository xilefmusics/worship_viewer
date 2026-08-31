---
name: create-idea
description: Create and index scored Worship Viewer idea documents from rough idea text. Use when capturing or promoting an idea under docs/issues.
---

# Create Idea

Turn the user's input into one concise, assessed idea document and add it to the
repository's idea index.

## Workflow

1. Read `docs/issues/templates/idea-template.md` and the `## Ideas` table in
   `docs/issues/Readme.md`.
2. If no idea text was provided, ask for it and stop.
3. Derive a concise, outcome-oriented title and a lowercase kebab-case slug.
   Create `docs/issues/ideas/<slug>.md`. Never overwrite an existing document;
   make the slug more specific when it already exists.
4. Follow the template structure. Replace every placeholder, give every
   assessment a 1-5 score and concise reason, and preserve the score scales.
   Keep `[← Back to issues README](../Readme.md)` as the first visible content
   after the YAML front matter.
   Populate the visible `## Assessment` table with each score as `<score> / 5`
   and the same reason used in metadata. Set `status` to `rough`,
   `last_reviewed` to today's date, and `owner` to `null` unless the user
   identifies one.
5. Classify the impact audience as `user`, `maintainer`, or `both`; classify the
   change as `new capability or area` or `improvement to an existing capability`;
   identify the benefiting personas; and estimate total effort including tests,
   documentation, migrations, and rollout.
6. Keep the body minimal and concrete. Use repository evidence when it changes
   the assessment. Record unresolved details as open questions instead of
   inventing facts.
7. Append one row to the Ideas table with a relative document link and the area,
   primary persona, impact audience, change type, clarity, impact, effort, and
   status. Preserve existing rows and the `## Rough Ideas` section unless the
   user explicitly asks to promote or remove a matching rough idea. Escape pipe
   characters that would break the Markdown table.
8. Validate the YAML front matter and confirm the visible assessment table and
   index row match its metadata. Report links to the created document and updated
   index. Do not create a git commit unless the user explicitly requests one.
