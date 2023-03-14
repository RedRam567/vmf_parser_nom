//! Helper methods that should be in [`nom`] already.

use nom::error::{ContextError, ErrorKind, ParseError};

/// Helper methods that should be in [`nom`] already.
pub trait NomErrExt<E> {
    /// Returns the inner error from a [`nom::Err`]. Panics if it is [`nom::Err::Incomplete`].
    fn unwrap_error(self) -> E;
}

impl<E> NomErrExt<E> for nom::Err<E> {
    fn unwrap_error(self) -> E {
        match self {
            nom::Err::Incomplete(needed) => panic!("unwrap_error on Incomplete: {needed:?}"),
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        }
    }
}

/// Helper methods that should be in [`nom`] already.
pub trait ParseErrorExt<I, E>
where
    Self: ParseError<I>,
{
    /// Create [`ParseError`] from input and context.
    fn from_context(input: I, ctx: &'static str) -> Self;

    /// Wrap in [`Error`](nom::Err::Error) variant of [`nom::Err`].
    fn into_err(self) -> nom::Err<Self> {
        nom::Err::Error(self)
    }

    /// Wrap in [`Result<nom::Err>`] error.
    fn into_err_error<T>(self) -> Result<T, nom::Err<Self>> {
        Err(self.into_err())
    }
}

impl<T, I, E> ParseErrorExt<I, E> for T
where
    T: ParseError<I> + ContextError<I>,
    I: Clone,
{
    /// Creates a [`ErrorKind::Fail`] error and appends context to it.
    fn from_context(input: I, ctx: &'static str) -> Self {
        ContextError::add_context(
            input.clone(),
            ctx,
            ParseError::from_error_kind(input, ErrorKind::Fail),
        )
    }
}

// create directly for verbose error without ErrorKind::Fail
// fn from_context(input: I, ctx: &'static str) -> Self {
//     Self { errors: vec![(input, VerboseErrorKind::Context(ctx))] }
// }