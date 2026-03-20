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
        description: "switch between views",
    },
    ShortcutEntry {
        keys: "1 / 2",
        description: "jump to Changes / Branches view",
    },
    ShortcutEntry {
        keys: "Left / h",
        description: "select previous repository tab",
    },
    ShortcutEntry {
        keys: "Right / l",
        description: "select next repository tab",
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
        keys: "b",
        description: "open branch switcher",
    },
    ShortcutEntry {
        keys: "n",
        description: "create a new branch",
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

pub const BRANCHES_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "Down / j",
        description: "select next branch",
    },
    ShortcutEntry {
        keys: "Up / k",
        description: "select previous branch",
    },
    ShortcutEntry {
        keys: "Enter",
        description: "switch to selected branch",
    },
    ShortcutEntry {
        keys: "n",
        description: "create a new branch",
    },
    ShortcutEntry {
        keys: "d",
        description: "delete selected branch",
    },
    ShortcutEntry {
        keys: "m",
        description: "merge selected branch into current",
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
