#[derive(Debug, PartialEq)]
pub enum LlmProvider {
    OpenAi,
    Anthropic,
    Google,
    Local,
}

impl LlmProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openai" => Some(Self::OpenAi),
            "anthropic" => Some(Self::Anthropic),
            "google" => Some(Self::Google),
            "local" => Some(Self::Local),
            _ => None,
        }
    }
}
