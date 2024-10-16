use std::{error, fmt, sync::Arc};

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub enum CouchError {
    /// A `CouchDB` operation failed, typically indicated by a specific HTTP error status that was returned.
    OperationFailed(ErrorDetails),
    /// Parsing of a JSON document failed.
    InvalidJson(ErrorMessage),
    /// The provided url is invalid.
    MalformedUrl(ErrorMessage),
    /// A design document could not be created.
    CreateDesignFailed(ErrorMessage),
}

#[derive(Debug, Clone)]
pub struct ErrorDetails {
    /// Some (bulk) transaction might return an id as part of the error
    pub id: Option<String>,
    /// HTTP Status Code
    pub status: http::StatusCode,
    /// Detailed error message
    pub message: String,
    upstream: Option<UpstreamError>,
}

#[derive(Debug, Clone)]
pub struct ErrorMessage {
    /// Detailed error message
    pub message: String,
    pub(crate) upstream: Option<UpstreamError>,
}

type UpstreamError = Arc<dyn error::Error + Send + Sync + 'static>;
pub type CouchResult<T> = Result<T, CouchError>;

impl CouchError {
    #[must_use]
    pub fn new(message: String, status: http::StatusCode) -> CouchError {
        CouchError::OperationFailed(ErrorDetails {
            id: None,
            message,
            status,
            upstream: None,
        })
    }

    #[must_use]
    pub fn new_with_id(id: Option<String>, message: String, status: http::StatusCode) -> CouchError {
        CouchError::OperationFailed(ErrorDetails {
            id,
            message,
            status,
            upstream: None,
        })
    }

    #[must_use]
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(http::StatusCode::NOT_FOUND)
    }

    #[must_use]
    pub fn status(&self) -> Option<http::StatusCode> {
        match self {
            CouchError::OperationFailed(details) => Some(details.status),
            _ => None,
        }
    }
}

pub trait CouchResultExt<T> {
    /// turns an Ok into an Ok(Some), a not-found into an Ok(None), otherwise it will return the error.
    fn into_option(self) -> CouchResult<Option<T>>;
}

impl<T> CouchResultExt<T> for CouchResult<T> {
    fn into_option(self) -> CouchResult<Option<T>> {
        match self {
            Ok(r) => Ok(Some(r)),
            Err(err) => {
                if err.is_not_found() {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }
}

impl fmt::Display for CouchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CouchError::OperationFailed(details) => {
                if let Some(id) = &details.id {
                    write!(f, "{} -> {}: {}", id, details.status, details.message)
                } else {
                    write!(f, "{}: {}", details.status, details.message)
                }
            }
            CouchError::InvalidJson(err) | CouchError::MalformedUrl(err) | CouchError::CreateDesignFailed(err) => {
                write!(f, "{}", err.message)
            }
        }
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CouchError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        match self {
            CouchError::OperationFailed(details) => details.upstream.as_ref().map(|e| &**e as _),
            CouchError::InvalidJson(err) | CouchError::MalformedUrl(err) | CouchError::CreateDesignFailed(err) => {
                err.upstream.as_ref().map(|e| &**e as _)
            }
        }
    }
}

impl std::convert::From<reqwest::Error> for CouchError {
    fn from(err: reqwest::Error) -> Self {
        CouchError::OperationFailed(ErrorDetails {
            id: None,
            status: err.status().unwrap_or(http::StatusCode::NOT_IMPLEMENTED),
            message: err.to_string(),
            upstream: Some(Arc::new(err)),
        })
    }
}

impl std::convert::From<serde_json::Error> for CouchError {
    fn from(err: serde_json::Error) -> Self {
        CouchError::InvalidJson(ErrorMessage {
            message: err.to_string(),
            upstream: Some(Arc::new(err)),
        })
    }
}

impl std::convert::From<url::ParseError> for CouchError {
    fn from(err: url::ParseError) -> Self {
        CouchError::MalformedUrl(ErrorMessage {
            message: err.to_string(),
            upstream: Some(Arc::new(err)),
        })
    }
}
