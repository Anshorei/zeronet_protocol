use std::hash::Hash;

/// Responses transmitted over an AsyncConnection can be matched
/// with their request by looking for messages where `m1.req_id() == m2.to()`.
/// Higher level implementors over AsyncConnection can use any method
/// to generate ID's of any type that implements Eq + Hash.
pub trait Requestable {
  type Key: PartialEq + Eq + Hash + Send + Clone;

  /// Returns the request's ID if it has one.
  fn req_id(&self) -> Option<Self::Key>;
  /// Returns the ID of the request responded to.
  fn to(&self) -> Option<Self::Key>;

  /// If the message has a request ID, it is a request.
  /// It is possible for a response to simultaneously be
  /// a request.
  fn is_request(&self) -> bool {
    match self.req_id() {
      Some(_) => true,
      None => false,
    }
  }

  /// If the message has a response ID, it is a response.
  /// It is possible for a response to simultaneously be
  /// a request.
  fn is_response(&self) -> bool {
    match self.to() {
      Some(_) => true,
      None => false,
    }
  }
}
