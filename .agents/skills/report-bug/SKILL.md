---
name: report-bug
description: Use when investigating, drafting, or filing a reproducible bug report for Worship Viewer. Follow the repository's GitHub bug template and label conventions.
---

# Report a bug

When asked to report a Worship Viewer defect, produce a concise, reproducible report that matches the project's current issue template.

## Prepare the report

- Read `.github/ISSUE_TEMPLATE/bug_report.md` and use its headings and context/environment tables. Fill every section; replace all placeholders, including the three acceptance-criteria checkboxes under **Spec for “fixed”**, with concrete, testable statements.
- Capture exact reproduction steps, actual behavior, expected behavior, and relevant evidence. For UI bugs, include a screenshot or measurements when available and authorized. Do not upload recordings, screenshots, logs, or other files unless the user authorized that sharing.
- Record deployment, account/role, affected song/set/asset, OS, browser, app version or commit, and mobile device details when relevant. Mark unknown values as unknown; do not guess.
- Separate observed facts from suspected causes. Put hypotheses under **Suspected area** and label them as unconfirmed.

## File the issue

- Create a GitHub issue only when the user asks to file or create one. If they ask only for a report, return the report without publishing it.
- Before creating an issue, search this repository for likely duplicates. If a clear duplicate exists, report its link instead of creating another issue unless the user directs otherwise.
- Apply the template's `bug` label. Add `frontend` or `backend` according to the affected implementation layer, and a component label such as `player` when it applies and exists. Do not label both layers unless the defect crosses them; do not invent labels.
- Do not assign people or milestones unless asked.
- After filing, verify the issue title, body, and labels, then give the user its link.
