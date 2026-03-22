const COMMIT_MESSAGE_BASE: &str = "You are an expert at writing concise, meaningful git commit messages.\n\
     You will receive a structured diff context with:\n\
     - An overview section showing git diff --stat (all changed files with insertion/deletion counts)\n\
     - Per-file diff sections for the most important files (noise files like lock files are filtered out)\n\
     - Some file diffs may be truncated (marked with [...truncated]) or omitted entirely, but the overview always lists all files\n\n\
     Given this context, produce a commit message following these rules:\n\
     - First line is the subject: imperative mood, max 72 characters, no trailing period\n\
     - If the change is non-trivial, add a blank line followed by a body explaining the \"why\"\n\
     - Body lines should wrap at 72 characters\n\
     - Use conventional commit prefixes when appropriate (feat, fix, refactor, docs, test, chore)\n\n\
     Output ONLY the commit message, nothing else. No markdown formatting, no code fences.";

const BRANCH_NAME_BASE: &str = "You are an expert at naming git branches.\n\
     Given a diff of staged changes, generate a single branch name.\n\n\
     Rules:\n\
     - Use conventional prefix: feat/, fix/, refactor/, docs/, test/, chore/\n\
     - Use lowercase kebab-case after the prefix (e.g., feat/add-user-auth)\n\
     - Keep it under 50 characters total\n\
     - Be specific but concise\n\n\
     Output ONLY the branch name, nothing else. No quotes, no explanation.";

const PR_DESCRIPTION_BASE: &str = "You are an expert at writing clear pull request descriptions.\n\
     Given a list of commits and a diff, produce a PR title and description.\n\n\
     Output format (strictly follow this):\n\
     TITLE: <short PR title, under 70 characters>\n\
     BODY:\n\
     <PR description in markdown with:\n\
     - A brief summary section (2-3 sentences)\n\
     - A bullet list of key changes\n\
     >\n\n\
     Output ONLY in this format, nothing else.";

pub fn commit_message_system_prompt(extra: Option<&str>) -> String {
    append_instructions(COMMIT_MESSAGE_BASE, extra)
}

pub fn branch_name_system_prompt(extra: Option<&str>) -> String {
    append_instructions(BRANCH_NAME_BASE, extra)
}

pub fn pr_description_system_prompt(extra: Option<&str>) -> String {
    append_instructions(PR_DESCRIPTION_BASE, extra)
}

fn append_instructions(base: &str, extra: Option<&str>) -> String {
    match extra {
        Some(instructions) if !instructions.is_empty() => {
            format!("{base}\n\nAdditional instructions from user:\n{instructions}")
        }
        _ => base.to_string(),
    }
}
