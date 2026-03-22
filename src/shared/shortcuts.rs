pub struct ShortcutEntry {
    pub keys: &'static str,
    pub description: &'static str,
}

pub const GLOBAL_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "?",
        description: "toggle keyboard shortcuts help",
    },
    ShortcutEntry {
        keys: "q",
        description: "quit the TUI",
    },
    ShortcutEntry {
        keys: "r",
        description: "refresh repositories and status",
    },
    ShortcutEntry {
        keys: "Tab / Shift+Tab",
        description: "switch between repositories",
    },
    ShortcutEntry {
        keys: "Left / Right",
        description: "switch between views",
    },
    ShortcutEntry {
        keys: "1-2",
        description: "jump to Changes / PR",
    },
    ShortcutEntry {
        keys: "Alt+1..9",
        description: "jump to repository by position",
    },
    ShortcutEntry {
        keys: ",",
        description: "open settings",
    },
];

pub const CHANGES_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "Down / j",
        description: "select next changed file",
    },
    ShortcutEntry {
        keys: "Up / k",
        description: "select previous changed file",
    },
    ShortcutEntry {
        keys: "Space",
        description: "toggle stage/unstage selected file",
    },
    ShortcutEntry {
        keys: "s",
        description: "stage selected file",
    },
    ShortcutEntry {
        keys: "u",
        description: "unstage selected file",
    },
    ShortcutEntry {
        keys: "S",
        description: "stage all changes",
    },
    ShortcutEntry {
        keys: "U",
        description: "unstage all changes",
    },
    ShortcutEntry {
        keys: "c",
        description: "open commit panel",
    },
    ShortcutEntry {
        keys: "a",
        description: "amend last commit",
    },
    ShortcutEntry {
        keys: "b",
        description: "open branches",
    },
    ShortcutEntry {
        keys: "n",
        description: "create a new branch",
    },
    ShortcutEntry {
        keys: "L",
        description: "open commit log",
    },
    ShortcutEntry {
        keys: "R",
        description: "create a GitHub repository",
    },
    ShortcutEntry {
        keys: "PageDown / Ctrl+d",
        description: "scroll diff preview down",
    },
    ShortcutEntry {
        keys: "PageUp / Ctrl+u",
        description: "scroll diff preview up",
    },
];

pub const PR_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "Down / j",
        description: "select next pull request",
    },
    ShortcutEntry {
        keys: "Up / k",
        description: "select previous pull request",
    },
    ShortcutEntry {
        keys: "Enter",
        description: "open PR in browser",
    },
    ShortcutEntry {
        keys: "r",
        description: "refresh pull requests",
    },
    ShortcutEntry {
        keys: "PageDown / Ctrl+d",
        description: "scroll PR detail down",
    },
    ShortcutEntry {
        keys: "PageUp / Ctrl+u",
        description: "scroll PR detail up",
    },
];

pub const MODAL_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "Enter",
        description: "confirm active modal or submit input",
    },
    ShortcutEntry {
        keys: "Esc",
        description: "close active modal",
    },
    ShortcutEntry {
        keys: "Backspace",
        description: "delete last character in text inputs",
    },
    ShortcutEntry {
        keys: "Ctrl+n",
        description: "insert newline in the commit editor",
    },
    ShortcutEntry {
        keys: "Ctrl+g",
        description: "generate commit message with AI",
    },
    ShortcutEntry {
        keys: "Ctrl+l",
        description: "login to GitHub Copilot",
    },
];
