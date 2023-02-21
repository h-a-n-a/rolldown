use once_cell::sync::Lazy;
use phf::{phf_set, Set};

pub static RESERVED_NAMES: Set<&'static str> = phf_set! {
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "NaN",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "var",
    "void",
    "while",
    "with",
    "yield",
};

fn starts_with_digit(s: &str) -> bool {
  s.chars().next().map_or(false, |c| c.is_ascii_digit())
}

fn need_escape(s: &str) -> bool {
  starts_with_digit(s) || RESERVED_NAMES.contains(s) || s == "arguments"
}

static ILLEGAL_CHARACTERS: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"[^\w$]").unwrap());

pub static CAPTURE_WORD_RE: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"-(\w)").unwrap());

pub fn make_legal(value: &str) -> String {
  let value = ILLEGAL_CHARACTERS.replace_all(value, "_");

  let ret = if need_escape(&value) {
    format!("_{}", value)
  } else {
    value.to_string()
  };

  if ret != value {
    tracing::warn!("illegal identifier: {}, replaced with {}", value, ret);
  }

  ret
}
