# Sidebar thread grouping — worktree path canonicalization

## Problem

Threads in the sidebar are grouped by their `folder_paths` (a `PathList` stored
in the thread metadata database). When a thread is created from a git worktree
checkout (e.g. `/Users/eric/repo/worktrees/zed/lasalle-lceljoj7/zed`), its
`folder_paths` records the worktree checkout path. Threads from different
checkouts of the same repos (different branches) have different raw paths and
don't group together.

## What we've done

### 1. `PathList` equality fix (PR #52052 — open, awaiting review)

**File:** `crates/util/src/path_list.rs`

Manual `PartialEq`/`Eq`/`Hash` impls that only compare the sorted `paths`
field, ignoring display order.

### 2. Canonical workspace grouping (this branch)

Replaced the old "absorption" model (worktree workspaces absorbed under main
repo via index-based tracking) with **canonical-key grouping**: workspaces that
share the same root repo paths are grouped under a single sidebar header.

#### Architecture

- **`build_worktree_root_mapping()`** — iterates ALL repos from all workspaces
  to build `HashMap<PathBuf, Arc<Path>>` mapping checkout paths → root repo
  paths. Uses all repos (not just root repos) for robustness when
  linked-worktree snapshots are temporarily incomplete.

- **`build_canonical_thread_index()`** — indexes all threads by their
  canonicalized `folder_paths` (checkout paths mapped to root repo paths).

- **`rebuild_contents()` flow:**
  1. Group workspaces by canonical key
  2. For each group: claim threads from canonical index, merge live info from
     all workspaces in the group, build thread entries with best-workspace
     selection (raw path match preferred)
  3. Historical groups: iterate unclaimed threads, group by raw `folder_paths`,
     create Closed project group sections

- **Worktree chips** — threads from single-root worktree checkouts that differ
  from the canonical key get a `{worktree-name}` chip via
  `linked_worktree_short_name`.

- **`Workspace::path_list()`** — moved from free function to method on
  `Workspace`.

- **`ProjectHeader.workspace`** is `Option<Entity<Workspace>>` to support
  closed historical group headers.

- **`find_current_workspace_for_path_list` /
  `find_open_workspace_for_path_list`** — canonicalize both sides before
  comparing.

- **`activate_archived_thread`** — when no matching workspace is found, saves
  metadata and sets `focused_thread` instead of opening a new workspace.

- **`prune_stale_worktree_workspaces`** — checks `all_workspace_roots` (from
  `workspace.root_paths()`) instead of git repo snapshots, so the check works
  even before git scan completes.

- **`thread_entry_from_metadata`** — extracted helper for building ThreadEntry.

- **`SidebarThreadMetadataStore::all_entries()`** — returns `&[ThreadMetadata]`
  for reference-based iteration.

## Remaining issues (priority order)

### 1. `save_thread` overwrites `folder_paths` on every thread mutation

**Severity: High — causes data loss**

`NativeAgent::save_thread()` (in `crates/agent/src/agent.rs`) fires on every
`cx.observe` of the thread entity. It always re-snapshots `folder_paths` from
the session's project's `visible_worktrees().abs_path()`. When a thread is
loaded in the wrong workspace (e.g. main repo instead of worktree checkout),
viewing the thread overwrites its `folder_paths` with the wrong paths,
permanently losing the worktree association.

**Fix needed:** Fix the loading side — when a thread is loaded (from sidebar
click, session restore, or archive restore), route it to a workspace whose raw
paths match its saved `folder_paths`. If no matching workspace exists, create
one. This way `save_thread` naturally preserves the correct paths.

Affected code paths:
- **Session restore:** `AgentPanel::load` in `crates/agent_ui/src/agent_panel.rs`
  (~L907-920) — loads the last active thread into whichever workspace is being
  restored, regardless of the thread's `work_dirs`
- **Sidebar click:** `confirm` / `render_thread` → `activate_thread` → loads in
  the `ThreadEntryWorkspace` which may be the wrong workspace (fallback to
  first in group)
- **Archive restore:** `activate_archived_thread` — currently just saves
  metadata and focuses, but clicking the resulting entry still routes through
  `open_workspace_and_activate_thread` → `find_existing_workspace`

### 2. Click-to-open from Closed groups goes through `find_existing_workspace`

When a user clicks a thread under a `Closed` historical group header,
`open_workspace_and_activate_thread` calls `open_paths` →
`find_existing_workspace`, which routes to an existing workspace that contains
the path instead of creating a new workspace tab.

**Fix:** Either pass `open_new_workspace: Some(true)` through the call chain,
or use a direct workspace creation path that bypasses `find_existing_workspace`.

### 3. Best-workspace selection is O(group_size) per thread

`group_workspaces.iter().find(|ws| ws.read(cx).path_list(cx) == row.folder_paths)`
scans all workspaces in the group for each thread. Should pre-build a
`HashMap<PathList, Entity<Workspace>>` per group for O(1) lookup.

### 4. Label allocation in historical group sort

`workspace_label_from_path_list` allocates a `SharedString` on every comparison
during the sort. Should cache labels before sorting.

### 5. Collapse state doesn't transfer between raw and canonical keys

If a user collapses a historical group (keyed by raw `folder_paths`), then opens
that workspace (which uses the canonical key), the collapse state doesn't
transfer. Minor UX issue.

### 6. Missing test coverage

- Clicking a thread in a historical (Closed) group
- The prune fix with `all_workspace_roots` vs snapshot-based check
- Multiple worktree checkouts grouped under one header (dedicated test)

### 7. Path set mutation (adding/removing folders)

When you add a folder to a project (e.g. adding `ex` to a `zed` workspace),
existing threads saved with `[zed]` don't match the new `[ex, zed]` path list.
Design decision still being discussed.

## Key code locations

- **Thread metadata storage:** `crates/agent_ui/src/thread_metadata_store.rs`
  - `SidebarThreadMetadataStore` — in-memory cache + SQLite DB
  - `threads_by_paths: HashMap<PathList, Vec<ThreadMetadata>>`
- **Sidebar rebuild:** `crates/sidebar/src/sidebar.rs`
  - `rebuild_contents()` — canonical-key grouping + historical groups
  - `build_worktree_root_mapping()` — worktree→root path map
  - `build_canonical_thread_index()` — threads indexed by canonical path
  - `canonicalize_path_list()` — maps a PathList through the root mapping
  - `thread_entry_from_metadata()` — helper for building ThreadEntry
  - `prune_stale_worktree_workspaces()` — uses `all_workspace_roots`
- **Thread saving:** `crates/agent/src/agent.rs`
  - `NativeAgent::save_thread()` — snapshots `folder_paths` on every mutation
- **Thread loading (session restore):** `crates/agent_ui/src/agent_panel.rs`
  - `AgentPanel::load` (~L907-920) — deserializes last active thread
- **Workspace opening:** `crates/workspace/src/workspace.rs`
  - `find_existing_workspace()` — dedup/routing that swallows worktree checkouts
  - `Workspace::new_local()` — creates workspace, canonicalizes paths
  - `Workspace::path_list()` — returns PathList from visible worktrees
- **Session restore:** `crates/workspace/src/workspace.rs`
  - `restore_multiworkspace()` — restores workspace tabs from session DB
- **PathList:** `crates/util/src/path_list.rs`

## Useful debugging queries

```sql
-- All distinct folder_paths in the sidebar metadata store
sqlite3 ~/Library/Application\ Support/Zed/db/0-{channel}/db.sqlite \
  "SELECT folder_paths, COUNT(*) FROM sidebar_threads GROUP BY folder_paths ORDER BY COUNT(*) DESC"

-- Find a specific thread
sqlite3 ~/Library/Application\ Support/Zed/db/0-{channel}/db.sqlite \
  "SELECT session_id, title, folder_paths FROM sidebar_threads WHERE title LIKE '%search term%'"

-- Check workspace session bindings
sqlite3 ~/Library/Application\ Support/Zed/db/0-{channel}/db.sqlite \
  "SELECT workspace_id, paths, session_id, window_id FROM workspaces WHERE paths LIKE '%search%' ORDER BY timestamp DESC"
```
