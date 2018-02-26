#[derive(Debug)]
/// A generic-error that contains the serialized error-kind, description, the position (file, line)
/// and an optional sub-error
pub struct WrappedError {
	pub kind_repr: String,
	pub description: String,
	pub file: &'static str,
	pub line: u32,
	pub sub_error: Option<std::rc::Rc<WrappedError>>
}
impl<T: std::fmt::Debug + Send> From<Error<T>> for WrappedError {
	fn from(error: Error<T>) -> Self {
		WrappedError {
			kind_repr: format!("{:?}", error.kind), description: error.description,
			file: error.file, line: error.line,
			sub_error: error.sub_error
		}
	}
}
impl ToString for WrappedError {
	/// Converts the error into a human-readable description ("pretty-print")
	fn to_string(&self) -> String {
		// Serialize error and sub-error (if any)
		let mut string = format!("({}) {} (at {}:{})", self.kind_repr, self.description, self.file, self.line);
		if let Some(ref sub_error) = self.sub_error { string += "\n  - "; string += &sub_error.to_string(); }
		string
	}
}
unsafe impl Send for WrappedError {}



#[derive(Debug)]
/// A typed-error that contains the error-kind, description, the position (file, line) and an
/// optional sub-error
pub struct Error<T: std::fmt::Debug + Send> {
	pub kind: T,
	pub description: String,
	pub file: &'static str,
	pub line: u32,
	pub sub_error: Option<std::rc::Rc<WrappedError>>
}
impl<T: std::fmt::Debug + Send> Error<T> {
	/// Creates a new error with an explicit description
	///
	/// _Note: This function is not intended for direct use; take a look at the `new_err!()`-macro
	/// instead_
	pub fn with_kind_desc<S: ToString>(kind: T, description: S, file: &'static str, line: u32) -> Self {
		Error{ kind, description: description.to_string(), file, line, sub_error: None }
	}
	/// Creates a new error
	///
	/// _Note: This function is not intended for direct use; take a look at the `new_err!()`-macro
	/// instead_
	pub fn with_kind(kind: T, file: &'static str, line: u32) -> Self {
		let description = format!("{:?}", kind);
		Error::with_kind_desc(kind, description, file, line)
	}
	
	/// Creates a new error with an explicit description and a sub-error
	///
	/// _Note: This function is not intended for direct use; take a look at the `rethrow_err!()`-
	/// macro instead_
	pub fn propagate_with_kind_desc<S: ToString>(kind: T, description: S, sub_error: WrappedError, file: &'static str, line: u32) -> Self {
		Error{ kind, description: description.to_string(), file, line, sub_error: Some(std::rc::Rc::new(sub_error)) }
	}
	/// Creates a new error with a sub-error
	///
	/// _Note: This function is not intended for direct use; take a look at the `rethrow_err!()`-
	/// macro instead_
	pub fn propagate_with_kind(kind: T, sub_error: WrappedError, file: &'static str, line: u32) -> Self {
		let description = format!("{:?}", kind);
		Error::propagate_with_kind_desc(kind, description, sub_error, file, line)
	}
	
	/// Creates a new error with the same kind and description as in the sub-error
	///
	/// _Note: This function is not intended for direct use; take a look at the `rethrow_err!()`-
	/// macro instead_
	pub fn propagate(sub_error: Error<T>, file: &'static str, line: u32) -> Self where T: Clone {
		Error::propagate_with_kind_desc(sub_error.kind.clone(), sub_error.description.clone(), sub_error.into(), file, line)
	}
}
impl<T: std::fmt::Debug + Send> ToString for Error<T> {
	/// Converts the error into a human-readable description ("pretty-print")
	fn to_string(&self) -> String {
		// Serialize error and sub-error (if any)
		let mut string = format!("({:?}): {} (at {}:{})", self.kind, self.description, self.file, self.line);
		if let Some(ref sub_error) = self.sub_error { string += "\n  - "; string += &sub_error.to_string(); }
		string
	}
}
unsafe impl<T: std::fmt::Debug + Send> Send for Error<T> {}



#[macro_export]
/// Creates a new error
///
/// Use `new_err!(error_kind)` to create an error with an automatically created description or use
/// `new_err!(error_kind, description)` to provide an explicit description
macro_rules! new_err {
    ($kind:expr, $description:expr) => ($crate::Error::with_kind_desc($kind, $description, file!(), line!()));
    ($kind:expr) => ($crate::Error::with_kind($kind, file!(), line!()));
}

#[macro_export]
/// Creates a new error and returns it (`return Err(created_error)`)
///
/// Use `throw_err!(error_kind)` to create an error with an automatically created description or use
/// `throw_err!(error_kind, description)` to provide an explicit description
macro_rules! throw_err {
    ($kind:expr, $description:expr) => (return Err($crate::Error::with_kind_desc($kind, $description, file!(), line!())));
    ($kind:expr) => (return Err($crate::Error::with_kind($kind, file!(), line!())));
}

#[macro_export]
/// Creates a new error with a sub-error and returns it (`return Err(created_error)`)
///
/// Use `rethrow_err!(error_kind, error)` to create an error with an automatically created
/// description or use
/// `rethrow_err!(error_kind, description, error)` to provide an explicit description
macro_rules! rethrow_err {
    ($kind:expr, $description:expr, $suberr:expr) =>
    	(return Err($crate::Error::propagate_with_kind_desc($kind, $description, $suberr.into(), file!(), line!())));
    ($kind:expr, $suberr:expr) =>
    	(return Err($crate::Error::propagate_with_kind($kind, $suberr.into(), file!(), line!())));
}

#[macro_export]
/// Runs an expression and returns either the unwrapped result or creates a new error with the
/// returned error as sub-error and returns the new error (`return Err(Error<T>)`)
///
/// Use `try_err!(expression)` to adopt the underlying error-kind and description or use
/// `try_err!(expression, error_kind)` to create an error with an automatically created description
/// or use
/// `try_err!(expression, error_kind, description)` to provide an explicit description
macro_rules! try_err {
	($code:expr, $kind:expr, $description:expr) => (match $code {
		Ok(result) => result,
		Err(error) => return Err($crate::Error::propagate_with_kind_desc($kind, $description, error.into(), file!(), line!()))
	});
	($code:expr, $kind:expr) => (match $code {
		Ok(result) => result,
		Err(error) => return Err($crate::Error::propagate_with_kind($kind, error.into(), file!(), line!()))
	});
	($code:expr) => (match $code {
		Ok(result) => result,
		Err(error) => return Err($crate::Error::propagate(error.into(), file!(), line!()))
	});
}

#[macro_export]
/// Runs an expression and returns either the unwrapped result or converts the error __into__ the
/// matching `Error<T>` (using a `From`-trait) and returns it (`return Err(Error<T>)`)
///
/// Use `try_convert_err!(expression)` to create an error with an automatically created description or
/// use `try_convert_err!(expression, description)` to provide an explicit description
macro_rules! try_convert_err {
    ($code:expr, $description:expr) => (match $code {
		Ok(result) => result,
		Err(error) => throw_err!(error.into(), $description)
	});
	($code:expr) => (match $code {
		Ok(result) => result,
		Err(error) => throw_err!(error.into())
	});
}