---
title: Parallel Agents - Zed
description: Run multiple AI agents in parallel across many projects in the same Zed window, managed through the Threads Panel.
---

# Parallel agents

You can run multiple agents at once, across different projects, in the same Zed window. The Threads Panel groups your threads by project so you can see what each agent is doing and switch between them. Each window still has one active project at a time, same as always.

## Threads panel {#threads-panel}

The Threads Panel lists your projects and the threads inside each one. Projects are collapsible groups. Active threads show their status, and clicking a thread opens it.

## Managing projects {#managing-projects}

### Window scoping {#window-scoping}

Each window has its own list of projects in the Threads Panel. You can have multiple windows open, each with different projects running agents.

Threads themselves are global (tied to your filesystem), but the project list is per-window.

If you try to open a folder that's already open in another window, Zed focuses that window. If the folder isn't open anywhere, it opens in the current window.

Opening a new window gives you an empty Threads Panel. Using the CLI (e.g. `zed my-project/`) focuses an existing window for that project, or opens a new one.

### Adding projects {#adding-projects}

Open a folder the way you normally would -- through the recent projects picker, by opening a new folder, or via the CLI. The project appears in the Threads Panel above your last opened project.

### Removing projects {#removing-projects}

Click the close button on a project's header to remove it from the Threads Panel. This doesn't delete the project's threads -- re-open the project to see them again.

You can't remove the currently active project. If you remove a project that has a running thread, that thread stops.

### Multi-folder projects {#multi-folder-projects}

When you add or remove folders from your current project, the Threads Panel updates accordingly. The new folder configuration is added as a separate project entry, so threads running against the previous configuration continue uninterrupted.

## Worktrees {#worktrees}

Select "New Worktree" from the thread dropdown to create a Git worktree for each root repository in the project. Each worktree thread appears under its parent project with a chip showing which worktree it's using. This keeps agent sessions isolated from each other.

> [!NOTE]
> Worktrees are treated as separate projects but are displayed under their main repository's project. You can have the main project open in one window and a linked worktree in another.

> [!NOTE]
> Worktrees only appear in the Threads Panel once they have at least one thread.

For worktrees created outside of Zed, open the folder and start a thread in the agent panel. The worktree chip appears automatically.

To use multiple worktrees from different repos in one thread (e.g. `zed/worktree-1` and `cloud/externally-created-worktree`), add them as folders to the same project with {#kb project::AddFolderToProject}, then start a new thread. The thread shows a worktree chip for each configured worktree.

To open worktrees in separate windows, open each folder in its own window.

## See also

- [Agent panel](./agent-panel.md): The main interface for interacting with agents
- [Tools](./tools.md): Built-in tools available to agents
- [Agent settings](./agent-settings.md): Configure agent behavior and model providers
