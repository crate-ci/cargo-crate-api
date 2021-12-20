#[derive(Clone, Debug)]
pub struct Error {
    kind: ErrorKind,
    context: String,
    source: Option<std::sync::Arc<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    pub fn new(kind: ErrorKind, context: impl std::fmt::Display) -> Self {
        Self {
            kind: kind,
            context: context.to_string(),
            source: None,
        }
    }

    pub fn set_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(std::sync::Arc::new(source));
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|s| s as &(dyn std::error::Error + 'static))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    ApiParse,
    Unknown,
}
