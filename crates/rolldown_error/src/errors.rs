use crate::Error;

/// A collection of rolldown [Error].
///
/// Yeah, this is just a wrapper of `Vec<Error>` but with a few promises:
///
/// [Errors] is never empty. You could only construct a `Errors` from a `Error`.
#[derive(Debug)]
pub struct Errors(Vec<Error>);

impl Errors {
  pub fn new(err: Error) -> Self {
    Self(vec![err])
  }

  pub fn push(&mut self, error: Error) {
    self.0.push(error);
  }

  pub fn into_vec(self) -> Vec<Error> {
    self.0
  }
}

impl Extend<Error> for Errors {
  fn extend<T: IntoIterator<Item = Error>>(&mut self, iter: T) {
    self.0.extend(iter)
  }
}

impl From<Error> for Errors {
  fn from(error: Error) -> Self {
    Self(vec![error])
  }
}
