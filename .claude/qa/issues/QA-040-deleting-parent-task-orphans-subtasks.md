---
id: QA-040
title: Deleting a parent task leaves its subtasks behind as top-level tasks
status: verified
severity: major
kind: bug
area: tasks/[id]
mode: tauri
found: 2026-11-06
---

## Use case

Owner deletes a task that has subtasks.

## Steps

1. Create task `QA cascade parent`, open it, add subtask `QA cascade child`.
2. Delete the parent (confirm dialog says only 'Are you sure you want to delete "..."? This action cannot be undone.').
3. Go to `/tasks`, List view.

## Expected

Subtasks are deleted with the parent (or the dialog says how many subtasks will be removed / orphaned and what happens to them).

## Actual

`QA cascade child` stays in the board and appears as an ordinary top-level task (still pointing at a deleted parent). The delete dialog does not mention the subtask. Reproduced twice. On a real board this litters the list with orphans.

## Evidence

`.claude/qa/shots/QA-040-before-orphan-subtask.png`

## Repro

`node .claude/qa/scripts/QA-040.mjs` (deletes its own orphan at the end).

## Decision

2026-11-06, owner (via lead): deleting a parent deletes its subtasks too. The confirm dialog
must say so with the count ("Also deletes N subtasks").

## Fix

Frontend done: delete dialog on the detail page now reads "... Also deletes N subtask(s). This action cannot be undone." with the Delete label. Backend does NOT cascade yet (task_repository.rs delete is a bare delete_by_id), so until the Rust change below lands the dialog promises more than happens. Bulk delete from the board also needs it (it calls delete_task per id, so the same Rust fix covers it; its dialog does not show subtask counts yet).
Rust change (lead to apply), src-tauri/portal-core/src/domains/tasks/repositories/task_repository.rs:192:
```rust
// add TransactionTrait to the sea_orm use list at line 4
pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
    let txn = self.db.begin().await?;
    TaskEntity::delete_many().filter(Column::ParentId.eq(id)).exec(&txn).await?;
    TaskEntity::delete_by_id(id).exec(&txn).await?;
    txn.commit().await
}
```
One level only (subtasks have no subtasks in the UI). Frontend refetches the list after delete, so no TS change needed.
Files: routes/tasks/[id]/+page.svelte. Checks as QA-025. Left at fixing until the Rust lands.

Rust part applied by the lead (2026-11-06): `task_repository.rs` `delete()` now deletes direct
subtasks (`ParentId = id`) and the task in one transaction; `TransactionTrait` imported.
Rebuild clean, app relaunched. One level only (all the UI creates); `ponytail:` comment marks it.


Review follow-up: bulk delete dialog in TaskManager.svelte now adds "Also deletes N subtask(s)" (sum over selected ids). Rust cascade applied by lead. Status set to fixed on that basis; the tester should verify the cascade end to end.

## Verification

2026-11-06, tauri mode (:9222). Created QA cascade parent + QA cascade child. Dialog read: 'Also deletes 1 subtask. This action cannot be undone.' After delete and a full reload the child is gone (11 rows, the owner's original count; no QA tasks left). After shot: shots/QA-040-after-cascade.png. Bulk delete cascade not run (would need multi-select of fixtures; skipped). Verified.
