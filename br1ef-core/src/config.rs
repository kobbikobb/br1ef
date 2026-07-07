#[derive(Debug, Clone)]
pub struct AppConfig {
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_username: String,
    pub imap_password: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
}

impl AppConfig {
    pub fn defaults() -> Self {
        Self {
            imap_host: "imap.gmail.com".into(),
            imap_port: 993,
            imap_username: "".into(),
            imap_password: "".into(),
            ollama_base_url: "http://localhost:11434".into(),
            ollama_model: "qwen2.5-coder:7b".into(),
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.imap_host.is_empty()
            && !self.imap_username.is_empty()
            && !self.imap_password.is_empty()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::defaults()
    }
}
