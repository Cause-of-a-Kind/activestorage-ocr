use crate::Args;

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub default_language: String,
    pub max_file_size: usize,
    #[allow(dead_code)]
    pub tessdata_path: Option<String>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            host: args.host,
            port: args.port,
            default_language: args.default_language,
            max_file_size: args.max_file_size,
            tessdata_path: args.tessdata_path,
        }
    }
}
