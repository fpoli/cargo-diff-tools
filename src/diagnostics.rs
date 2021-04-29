use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
pub struct Diagnostic {
    pub message: Option<Message>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
pub struct Message {
    pub level: Level,
    pub rendered: String,
    pub spans: Vec<Span>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Help,
    Note,
    Warning,
    Error,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
pub struct Span {
    pub file_name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
}

impl Message {
    /// Return the first primary span, if there is any.
    pub fn primary_span(&self) -> Option<&Span> {
        self.spans.iter().find(|s| s.is_primary)
    }
}
