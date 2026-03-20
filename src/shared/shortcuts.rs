pub struct ShortcutSection {
    pub title: &'static str,
    pub entries: &'static [ShortcutEntry],
}

pub struct ShortcutEntry {
    pub keys: &'static str,
    pub description: &'static str,
}

pub const SHORT_HELP: &str = "? help  q quit  h/l repos  j/k files  s/u stage  S/U all  b switch  n branch  c commit  r refresh";

const GLOBAL_SHORTCUTS: &[ShortcutEntry] = &[
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
];

const NAVIGATION_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "h / Left",
        description: "select previous repository tab",
    },
    ShortcutEntry {
        keys: "l / Right",
        description: "select next repository tab",
    },
    ShortcutEntry {
        keys: "j / Down",
        description: "select next changed file or branch",
    },
    ShortcutEntry {
        keys: "k / Up",
        description: "select previous changed file or branch",
    },
];

const CHANGE_SHORTCUTS: &[ShortcutEntry] = &[
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
];

const BRANCH_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "b",
        description: "open branch switcher",
    },
    ShortcutEntry {
        keys: "n",
        description: "create and switch to a new branch",
    },
];

const COMMIT_SHORTCUTS: &[ShortcutEntry] = &[
    ShortcutEntry {
        keys: "c",
        description: "open commit panel",
    },
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
];

pub const HELP_SECTIONS: &[ShortcutSection] = &[
    ShortcutSection {
        title: "Global",
        entries: GLOBAL_SHORTCUTS,
    },
    ShortcutSection {
        title: "Navigation",
        entries: NAVIGATION_SHORTCUTS,
    },
    ShortcutSection {
        title: "Changes",
        entries: CHANGE_SHORTCUTS,
    },
    ShortcutSection {
        title: "Branches",
        entries: BRANCH_SHORTCUTS,
    },
    ShortcutSection {
        title: "Commit and Modals",
        entries: COMMIT_SHORTCUTS,
    },
];
