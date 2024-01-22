pub enum CratesError {
    Reqwest(reqwest::Error),
    /// Crate not found
    CrateNotFound(Option<String>),
    InvalidQueryInDocLookup,
    CratesIoParseJson(reqwest::Error),
}

impl From<reqwest::Error> for CratesError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
