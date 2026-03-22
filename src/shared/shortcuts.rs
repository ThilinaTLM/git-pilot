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
        keys: "tab / shift+tab",
        description: "switch between repositories",
    },
    ShortcutEntry {
        keys: "left / right",
        description: "switch between views",
    },
    ShortcutEntry {
        keys: "1-2",
        description: "jump to Changes / PR",
    },
    ShortcutEntry {
        keys: "alt+1..9",
        description: "jump to repository by position",
    },
    ShortcutEntry {
        keys: ",",
        description: "open settings",
    },
];

pub const CHANGES_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "down / j",
        description: "select next changed file",
    },
    ShortcutEntry {
        keys: "up / k",
        description: "select previous changed file",
    },
    ShortcutEntry {
        keys: "space",
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
        keys: "pagedown / ctrl+d",
        description: "scroll diff preview down",
    },
    ShortcutEntry {
        keys: "pageup / ctrl+u",
        description: "scroll diff preview up",
    },
];

pub const PR_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "down / j",
        description: "select next pull request",
    },
    ShortcutEntry {
        keys: "up / k",
        description: "select previous pull request",
    },
    ShortcutEntry {
        keys: "enter",
        description: "open PR in browser",
    },
    ShortcutEntry {
        keys: "r",
        description: "refresh pull requests",
    },
    ShortcutEntry {
        keys: "pagedown / ctrl+d",
        description: "scroll PR detail down",
    },
    ShortcutEntry {
        keys: "pageup / ctrl+u",
        description: "scroll PR detail up",
    },
];

pub const MODAL_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "enter",
        description: "confirm active modal or submit input",
    },
    ShortcutEntry {
        keys: "esc",
        description: "close active modal",
    },
    ShortcutEntry {
        keys: "backspace",
        description: "delete last character in text inputs",
    },
    ShortcutEntry {
        keys: "ctrl+n",
        description: "insert newline in the commit editor",
    },
    ShortcutEntry {
        keys: "ctrl+g",
        description: "generate commit message with AI",
    },
    ShortcutEntry {
        keys: "ctrl+l",
        description: "login to GitHub Copilot",
    },
];
