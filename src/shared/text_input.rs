/// A text input buffer with cursor position tracking.
///
/// Supports cursor movement (left/right/home/end), word-level navigation,
/// insert at cursor, delete at cursor (backspace/delete), and select-all.
#[derive(Clone, Debug, Default)]
pub struct TextInput {
    content: String,
    /// Byte offset of cursor position within `content`.
    cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn set_content(&mut self, text: String) {
        self.cursor = text.len();
        self.content = text;
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        // Find the previous char boundary
        let prev = self.prev_char_boundary();
        self.content.drain(prev..self.cursor);
        self.cursor = prev;
    }

    pub fn delete(&mut self) {
        if self.cursor >= self.content.len() {
            return;
        }
        let next = self.next_char_boundary();
        self.content.drain(self.cursor..next);
    }

    pub fn move_left(&mut self) {
        self.cursor = self.prev_char_boundary();
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            self.cursor = self.next_char_boundary();
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.content.len();
    }

    pub fn move_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let bytes = self.content.as_bytes();
        let mut pos = self.prev_char_boundary();
        // Skip whitespace/punctuation going left
        while pos > 0 && !bytes[pos - 1].is_ascii_alphanumeric() {
            pos = Self::prev_boundary(&self.content, pos);
        }
        // Skip word chars going left
        while pos > 0 && bytes[pos - 1].is_ascii_alphanumeric() {
            pos = Self::prev_boundary(&self.content, pos);
        }
        self.cursor = pos;
    }

    pub fn move_word_right(&mut self) {
        let len = self.content.len();
        if self.cursor >= len {
            return;
        }
        let bytes = self.content.as_bytes();
        let mut pos = self.next_char_boundary();
        // Skip word chars going right
        while pos < len && bytes[pos].is_ascii_alphanumeric() {
            pos = Self::next_boundary(&self.content, pos);
        }
        // Skip whitespace/punctuation going right
        while pos < len && !bytes[pos].is_ascii_alphanumeric() {
            pos = Self::next_boundary(&self.content, pos);
        }
        self.cursor = pos;
    }

    /// Delete from cursor to end of the current word.
    pub fn delete_word_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = self.cursor;
        self.move_word_left();
        self.content.drain(self.cursor..start);
    }

    /// Returns (text_before_cursor, text_after_cursor) for rendering.
    pub fn split_at_cursor(&self) -> (&str, &str) {
        (&self.content[..self.cursor], &self.content[self.cursor..])
    }

    /// Check if cursor is at end of content.
    pub fn cursor_at_end(&self) -> bool {
        self.cursor >= self.content.len()
    }

    // -- helpers --

    fn prev_char_boundary(&self) -> usize {
        Self::prev_boundary(&self.content, self.cursor)
    }

    fn next_char_boundary(&self) -> usize {
        Self::next_boundary(&self.content, self.cursor)
    }

    fn prev_boundary(s: &str, pos: usize) -> usize {
        let mut p = pos.saturating_sub(1);
        while p > 0 && !s.is_char_boundary(p) {
            p -= 1;
        }
        p
    }

    fn next_boundary(s: &str, pos: usize) -> usize {
        let mut p = pos + 1;
        while p < s.len() && !s.is_char_boundary(p) {
            p += 1;
        }
        p
    }
}

impl std::fmt::Display for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_cursor_movement() {
        let mut input = TextInput::new();
        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.content(), "hi");
        assert_eq!(input.cursor(), 2);

        input.move_left();
        assert_eq!(input.cursor(), 1);

        input.insert_char('!');
        assert_eq!(input.content(), "h!i");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn backspace_at_cursor() {
        let mut input = TextInput::new();
        input.set_content("hello".into());
        input.cursor = 3; // after "hel"
        input.backspace();
        assert_eq!(input.content(), "helo");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn delete_at_cursor() {
        let mut input = TextInput::new();
        input.set_content("hello".into());
        input.cursor = 2; // after "he"
        input.delete();
        assert_eq!(input.content(), "helo");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn home_and_end() {
        let mut input = TextInput::new();
        input.set_content("test".into());
        input.move_home();
        assert_eq!(input.cursor(), 0);
        input.move_end();
        assert_eq!(input.cursor(), 4);
    }

    #[test]
    fn word_movement() {
        let mut input = TextInput::new();
        input.set_content("hello world foo".into());
        input.move_word_left();
        assert_eq!(input.cursor(), 12); // before "foo"
        input.move_word_left();
        assert_eq!(input.cursor(), 6); // before "world"
        input.move_word_right();
        assert_eq!(input.cursor(), 12); // after "world "
    }

    #[test]
    fn split_at_cursor() {
        let mut input = TextInput::new();
        input.set_content("hello".into());
        input.cursor = 3;
        let (before, after) = input.split_at_cursor();
        assert_eq!(before, "hel");
        assert_eq!(after, "lo");
    }

    #[test]
    fn delete_word_back() {
        let mut input = TextInput::new();
        input.set_content("hello world".into());
        input.delete_word_back();
        assert_eq!(input.content(), "hello ");
    }
}
