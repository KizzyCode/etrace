/// A generic-error that contains the serialized error-kind, description, the position (file, line)
/// and an optional sub-error
#[derive(Debug, Clone)]
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
impl std::fmt::Display for WrappedError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}: {} (at {}:{})", self.kind_repr, self.description, self.file, self.line)?;
		if let Some(ref sub_error) = self.sub_error { write!(f, "\n  - {}", sub_error.to_string())?; }
		Ok(())
	}
}
impl std::error::Error for WrappedError {
	fn description(&self) -> &str { self.description.as_str() }
	
	fn cause(&self) -> Option<&std::error::Error> {
		self.sub_error.as_ref().and_then(|e| {
			let sub_error: &std::error::Error = e.as_ref();
			Some(sub_error)
		})
	}
}
unsafe impl Send for WrappedError {}


/// A typed-error that contains the error-kind, description, the position (file, line) and an
/// optional sub-error
#[derive(Debug)]
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
impl<T: std::fmt::Debug + Send> std::fmt::Display for Error<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}: {} (at {}:{})", self.kind, self.description, self.file, self.line)?;
		if let Some(ref sub_error) = self.sub_error { write!(f, "\n  - {}", sub_error.to_string())?; }
		Ok(())
	}
}
impl<T: std::fmt::Debug + Send> std::error::Error for Error<T> {
	fn description(&self) -> &str { self.description.as_str() }
}
unsafe impl<T: std::fmt::Debug + Send> Send for Error<T> {}


/// Creates a new error
///
/// Use `new_err!(kind)` to create an error with an automatically created description or use
/// `new_err!(kind, description)` to provide an explicit description
#[macro_export]
macro_rules! new_err {
	($kind:expr, $description:expr) => ($crate::Error::with_kind_desc($kind, $description, file!(), line!()));
	($kind:expr) => ($crate::Error::with_kind($kind, file!(), line!()));
}

/// Creates a new error containing the underlaying error
///
/// Use `new_err_with(error)` to create an error with the same kind and an automatic description
/// or use
/// `new_err_with(kind, error)` to provide a new error-kind or use
/// `new_err_with(kind, description, error)` to provide a new error-kind with an explicit
/// description
#[macro_export]
macro_rules! new_err_with {
	($kind:expr, $description:expr, $suberr:expr) =>
		($crate::Error::propagate_with_kind_desc($kind, $description, $suberr.into(), file!(), line!()));
	($kind:expr, $suberr:expr) =>
		($crate::Error::propagate_with_kind($kind, $suberr.into(), file!(), line!()));
	($suberr:expr) =>
		($crate::Error::propagate($suberr.into(), file!(), line!()))
}

/// Creates a new error by converting `kind` __into__ the matching `Error<T>` (using a `From`-trait)
/// and returns it (`return Err(Error<T>)`)
///
/// Use `new_err_from!(kind)` to create an error with an automatically created description or use
/// `new_err_from!(kind, description)` to provide an explicit description
#[macro_export]
macro_rules! new_err_from {
	($kind:expr, $description:expr) => (new_err!($kind.into(), $description));
	($kind:expr) => (new_err!($kind.into()));
}

/// Creates a new error and returns it (`return Err(created_error)`)
///
/// Use `throw_err!(kind)` to create an error with an automatically created description or use
/// `throw_err!(kind, description)` to provide an explicit description
#[macro_export]
macro_rules! throw_err {
	($kind:expr, $description:expr) => (return Err(new_err!($kind, $description)));
	($kind:expr) => (return Err(new_err!($kind)));
}

/// Creates a new error with a sub-error and returns it (`return Err(created_error)`)
///
/// Use `rethrow_err!(error)` to create an error with the same kind or use
/// `rethrow_err!(kind, error)` to provide a new error-kind or use
/// `rethrow_err!(kind, description, error)` to provide a new error-kind with an explicit
/// description
#[macro_export]
macro_rules! rethrow_err {
	($kind:expr, $description:expr, $suberr:expr) => (return Err(new_err_with!($kind, $description, $suberr)));
	($kind:expr, $suberr:expr) => (return Err(new_err_with!($kind, $suberr)));
	($suberr:expr) => (return Err(new_err_with!($suberr)));
}

/// Runs an expression and returns either the unwrapped result or creates a new error with the
/// returned error as sub-error and returns the new error (`return Err(Error<T>)`)
///
/// Use `try_err!(expression)` to adopt the underlying error-kind and description or use
/// `try_err!(expression, kind)` to create an error with an automatically created description
/// or use
/// `try_err!(expression, kind, description)` to provide an explicit description
#[macro_export]
macro_rules! try_err {
	($code:expr, $kind:expr, $description:expr) => (match $code {
		Ok(result) => result,
		Err(error) => rethrow_err!($kind, $description, error)
	});
	($code:expr, $kind:expr) => (match $code {
		Ok(result) => result,
		Err(error) => rethrow_err!($kind, error)
	});
	($code:expr) => (match $code {
		Ok(result) => result,
		Err(error) => rethrow_err!(error)
	});
}

/// Runs an expression and returns either the unwrapped result or converts the error __into__ the
/// matching `Error<T>` (using a `From`-trait) and returns it (`return Err(Error<T>)`)
///
/// Use `try_err_from!(expression)` to create an error with an automatically created description or
/// use `try_err_from!(expression, description)` to provide an explicit description
#[macro_export]
macro_rules! try_err_from {
	($code:expr, $description:expr) => (match $code {
		Ok(result) => result,
		Err(error) => throw_err!(error.into(), $description)
	});
	($code:expr) => (match $code {
		Ok(result) => result,
		Err(error) => throw_err!(error.into())
	});
}

/// Runs `$code` and returns either the unwrapped result or binds the error to `$err` and executes
/// `$or` (or can then access the error using the identifier passed as `$err`)
///
/// Example:
/// ```
/// // This code either prints the error and exits (if error) or prints the result (if ok)
/// let unwrapped = ok_or!(result, example_error_identifier, {
/// 	eprintln!("Fatal error: \"{}\"", example_error_identifier);
/// 	std::process::exit(1);
/// });
/// println!("Result: \"{}\"", unwrapped);
/// ```
#[macro_export]
macro_rules! ok_or {
	($code:expr, $err:ident, $or:expr) => (match $code {
		Ok(result) => result,
		Err($err) => $or
	});
	($code:expr, $or:expr) => (match $code {
		Ok(result) => result,
		Err(_) => $or
	});
}

/// Runs an expression and returns either the unwrapped result or executes `$or`
#[macro_export]
macro_rules! some_or {
	($code:expr, $or:expr) => (match $code {
		Some(result) => result,
		None => $or
	});
}