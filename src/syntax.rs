use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    PlainText,
    Rust,
    Python,
    JavaScript,
}

#[derive(Debug, Clone)]
pub struct SyntaxHighlighter {
    pub language: Language,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        SyntaxHighlighter {
            language: Language::PlainText,
        }
    }

    pub fn detect_language(filename: &str) -> Language {
        if filename.ends_with(".rs") {
            Language::Rust
        } else if filename.ends_with(".py") {
            Language::Python
        } else if filename.ends_with(".js") || filename.ends_with(".jsx") || filename.ends_with(".ts") || filename.ends_with(".tsx") {
            Language::JavaScript
        } else {
            Language::PlainText
        }
    }

    pub fn highlight_line(&self, line: &str) -> Vec<(String, Color)> {
        match self.language {
            Language::PlainText => vec![(line.to_string(), Color::default())],
            Language::Rust => self.highlight_rust(line),
            Language::Python => self.highlight_python(line),
            Language::JavaScript => self.highlight_javascript(line),
        }
    }

    fn highlight_rust(&self, line: &str) -> Vec<(String, Color)> {
        let keywords = ["fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod", "crate", "self", "super", "if", "else", "match", "for", "while", "loop", "return", "break", "continue", "async", "await", "move", "ref", "where", "type", "const", "static", "unsafe", "extern"];
        let types = ["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc", "Arc"];
        
        self.highlight_with_rules(line, &keywords, &types)
    }

    fn highlight_python(&self, line: &str) -> Vec<(String, Color)> {
        let keywords = ["def", "class", "if", "elif", "else", "for", "while", "return", "import", "from", "as", "try", "except", "finally", "with", "yield", "lambda", "pass", "break", "continue", "and", "or", "not", "in", "is", "None", "True", "False", "self", "print"];
        let types = ["int", "float", "str", "bool", "list", "dict", "tuple", "set", "None", "True", "False"];
        
        self.highlight_with_rules(line, &keywords, &types)
    }

    fn highlight_javascript(&self, line: &str) -> Vec<(String, Color)> {
        let keywords = ["function", "const", "let", "var", "if", "else", "for", "while", "return", "import", "export", "from", "class", "extends", "new", "this", "async", "await", "try", "catch", "finally", "throw", "typeof", "instanceof", "in", "of", "null", "undefined", "true", "false"];
        let types = ["Number", "String", "Boolean", "Array", "Object", "Function", "Promise", "Map", "Set", "null", "undefined", "true", "false"];
        
        self.highlight_with_rules(line, &keywords, &types)
    }

    fn highlight_with_rules(&self, line: &str, keywords: &[&str], types: &[&str]) -> Vec<(String, Color)> {
        let mut result = Vec::new();
        let mut chars = line.chars().peekable();
        let mut current = String::new();
        
        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                current.push(ch);
                chars.next();
            } else {
                if !current.is_empty() {
                    let color = if keywords.contains(&current.as_str()) {
                        Color::from_rgb(0.8, 0.4, 0.8) // Purple for keywords
                    } else if types.contains(&current.as_str()) {
                        Color::from_rgb(0.2, 0.7, 0.7) // Cyan for types
                    } else {
                        Color::default() // Default color
                    };
                    result.push((current.clone(), color));
                    current.clear();
                }
                
                // Handle comments
                if ch == '/' && chars.clone().nth(1) == Some('/') {
                    result.push((line[line.find('/').unwrap()..].to_string(), Color::from_rgb(0.4, 0.6, 0.4))); // Green for comments
                    break;
                }
                
                // Handle strings
                if ch == '"' || ch == '\'' {
                    let quote = ch;
                    current.push(ch);
                    chars.next();
                    while let Some(&next_ch) = chars.peek() {
                        current.push(next_ch);
                        chars.next();
                        if next_ch == quote {
                            break;
                        }
                    }
                    result.push((current.clone(), Color::from_rgb(0.6, 0.8, 0.4))); // Yellow for strings
                    current.clear();
                } else {
                    current.push(ch);
                    chars.next();
                    result.push((current.clone(), Color::default()));
                    current.clear();
                }
            }
        }
        
        if !current.is_empty() {
            let color = if keywords.contains(&current.as_str()) {
                Color::from_rgb(0.8, 0.4, 0.8)
            } else if types.contains(&current.as_str()) {
                Color::from_rgb(0.2, 0.7, 0.7)
            } else {
                Color::default()
            };
            result.push((current, color));
        }
        
        result
    }
}
