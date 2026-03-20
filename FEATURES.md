# Features

## Multi-Repository Management
- Automatic discovery of Git repositories in the current directory and subdirectories
- Tab-based navigation between multiple repositories (Left/Right or h/l)

## View System
- **Changes View** (1) — file staging, diff preview, and commit workflow
- **Branches View** (2) — full branch management with list, details, and actions
- Switch views with Tab/Shift+Tab or number keys (1, 2)
- View selector bar with future placeholders for Log and Remotes

## File Staging & Diff Preview
- Grouped file list: Staged, Unstaged, and Untracked sections
- Stage/unstage individual files (s/u) or toggle with Space
- Stage/unstage all files at once (S/U)
- Syntax-highlighted diff preview with scrollable output (PageDown/PageUp or Ctrl+d/Ctrl+u)

## Branch Management
- Switch branches via quick-switch modal (b) or Branches view (Enter)
- Create new branches (n)
- Delete branches with current-branch guard (d)
- Merge branches into current (m)
- Current branch indicator in lists

## Commit Workflow
- Multi-line commit message editor (c)
- Subject/body separation with paragraph support (Ctrl+n for newline)
- Staged-changes validation before committing

## Smart Contextual Suggestions
- Footer displays context-aware action hints based on current state
- Suggestions adapt to active view, modal state, and working tree status

## Keyboard Navigation
- Arrow keys as primary navigation, vim keys (j/k/h/l) as secondary
- View-aware help overlay with grouped shortcut sections (?)
- Three-tier key dispatch: modal → global → view-specific

## UI
- Dark color theme with accent highlighting
- Modal overlays for branch switching, branch creation, and commits
- Flash messages for operation feedback (info and error)
- Status bar showing current branch, branch count, and changed file count
