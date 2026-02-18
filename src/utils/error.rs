use std::{convert, fmt};

use axum::response;
use tracing_error::SpanTrace;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[must_use]
pub struct Error {
    span_trace: SpanTrace,

    inner: anyhow::Error,
}

impl Error {
    pub fn new(inner: anyhow::Error) -> Self {
        let span_trace = SpanTrace::capture();

        Self { span_trace, inner }
    }

    pub fn context<C>(self, context: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        let inner = self.inner.context(context);

        Self { inner, ..self }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return f
                .debug_struct("Error")
                .field("inner", &self.inner)
                .field("span_trace", &self.span_trace)
                .finish();
        }

        write!(f, "{:?}\n\nSpan traces:\n{}", self.inner, self.span_trace)
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        let span_trace = SpanTrace::capture();
        let inner = anyhow::Error::from(error);
        Self { span_trace, inner }
    }
}

impl tracing_error::ExtractSpanTrace for Error {
    fn span_trace(&self) -> Option<&SpanTrace> {
        Some(&self.span_trace)
    }
}

impl response::IntoResponse for Error {
    fn into_response(self) -> response::Response {
        todo!()
    }
}

pub trait Context<T, E> {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: fmt::Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T> Context<T, convert::Infallible> for Option<T> {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_or_else(
            || Err(must_use(Error::new(::anyhow::format_err!("{context}")))),
            |ok| Ok(ok),
        )
    }

    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_or_else(
            || Err(must_use(Error::new(::anyhow::format_err!("{}", context())))),
            |ok| Ok(ok),
        )
    }
}

impl<T, E> Context<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Error::new(anyhow::Error::new(error).context(context))),
        }
    }

    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Error::new(anyhow::Error::new(error).context(context()))),
        }
    }
}

impl<T> Context<T, Error> for Result<T, Error> {
    fn context<C>(self, context: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context(context)),
        }
    }

    fn with_context<C, F>(self, context: F) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context(context())),
        }
    }
}

#[doc(hidden)]
#[inline]
#[cold]
pub const fn must_use(error: Error) -> Error {
    error
}
