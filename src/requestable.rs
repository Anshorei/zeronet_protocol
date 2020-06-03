pub trait Requestable {
	fn req_id(&self) -> Option<usize>;
	fn to(&self) -> Option<usize>;

	fn is_request(&self) -> bool {
		match self.req_id() {
			Some(_) => true,
			_ => false,
		}
	}

	fn is_response(&self) -> bool {
		match self.to() {
			Some(_) => true,
			_ => false,
		}
	}
}
