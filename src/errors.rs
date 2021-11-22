/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use std::fmt;

/// A position in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    /// File name.
    pub filename: String,
    /// Row.
    pub line: usize,
    /// Column.
    pub column: usize,
}
impl Position {
    /// Constructor.
    pub fn new(filename: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            filename: filename.into(),
            line,
            column,
        }
    }
}

/// Alias for operation names ([`String`]).
pub type OperationName = String;
/// Alias for parameter names ([`String`]).
pub type ParameterName = String;
/// Alias for error explanations ([`String`]).
pub type Explanation = String;

/// Errors over the parameters of an operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterError {
    /// Type-checking error.
    UnexpectedType,
    /// Unsupported feature.
    NotSupported,
    /// Parameter is out of range.
    OutOfRange,
}

/// Errors over operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationError {
    /// Parameter-level error.
    Parameter(ParameterName, ParameterError),
    /// Arity error.
    TooManyParameters,
    /// Parameters do not make sense.
    LogicErrorInParameters(&'static str),
    /// Some required parameter is missing.
    MissingRequiredParameters,
    /// Missing block.
    MissingBlock,
    /// Nested compilation error inside an operation.
    Nested(Box<CompileError>),
    /// Operation size error.
    NotFitInSlice,
}

/// Top-level compile error.
///
/// **NB**: all constructors set the name of the file of the error position to the empty string.
/// Use [`Self::with_filename`] to specify the file name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// Syntax error.
    Syntax(Position, Explanation),
    /// Unknown operation.
    UnknownOperation(Position, OperationName),
    /// Operation-level error.
    Operation(Position, OperationName, OperationError),
}

impl CompileError {
    /// Creates a syntax error.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn syntax<S: ToString>(line: usize, column: usize, explanation: S) -> Self {
        CompileError::Syntax(Position::new("", line, column), explanation.to_string())
    }
    /// Creates an unknown operation error.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn unknown<S: ToString>(line: usize, column: usize, name: S) -> Self {
        CompileError::UnknownOperation(Position::new("", line, column), name.to_string())
    }
    /// Creates an operation-level error.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn operation<S: ToString>(
        line: usize,
        column: usize,
        name: S,
        error: OperationError,
    ) -> Self {
        CompileError::Operation(Position::new("", line, column), name.to_string(), error)
    }

    /// Some parameters are missing.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn missing_params<S: ToString>(line: usize, column: usize, name: S) -> Self {
        CompileError::Operation(
            Position::new("", line, column),
            name.to_string(),
            OperationError::MissingRequiredParameters,
        )
    }
    /// A block is missing.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn missing_block<S: ToString>(line: usize, column: usize, name: S) -> Self {
        CompileError::Operation(
            Position::new("", line, column),
            name.to_string(),
            OperationError::MissingBlock,
        )
    }
    /// Operation was given too many parameters.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn too_many_params<S: ToString>(line: usize, column: usize, name: S) -> Self {
        CompileError::Operation(
            Position::new("", line, column),
            name.to_string(),
            OperationError::TooManyParameters,
        )
    }

    /// Out of range parameter.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn out_of_range<S1: ToString, S2: ToString>(
        line: usize,
        column: usize,
        name: S1,
        param: S2,
    ) -> Self {
        let operation = OperationError::Parameter(param.to_string(), ParameterError::OutOfRange);
        CompileError::Operation(Position::new("", line, column), name.to_string(), operation)
    }

    /// Unexpected type error.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn unexpected_type<S1: ToString, S2: ToString>(
        line: usize,
        column: usize,
        name: S1,
        param: S2,
    ) -> Self {
        let operation =
            OperationError::Parameter(param.to_string(), ParameterError::UnexpectedType);
        CompileError::operation(line, column, name.to_string(), operation)
    }
    /// Logic error.
    ///
    /// Sets the name of the file of the error position as the empty string.
    pub fn logic_error<S: ToString>(
        line: usize,
        column: usize,
        name: S,
        error: &'static str,
    ) -> Self {
        let operation = OperationError::LogicErrorInParameters(error);
        CompileError::operation(line, column, name.to_string(), operation)
    }

    /// Filename accessor.
    pub fn filename(&self) -> &String {
        match self {
            Self::Syntax(pos, _) => &pos.filename,
            Self::UnknownOperation(pos, _) => &pos.filename,
            Self::Operation(pos, _, _) => &pos.filename,
        }
    }

    /// Sets the filename.
    pub fn with_filename(mut self, filename: String) -> Self {
        match self {
            Self::Syntax(ref mut pos, _) => {
                pos.filename = filename;
            }
            Self::UnknownOperation(ref mut pos, _) => {
                pos.filename = filename;
            }
            Self::Operation(ref mut pos, _, _) => {
                pos.filename = filename;
            }
        };
        self
    }
}

/// Turns itself into a parameter error for an operation.
pub trait ToOperationParameterError<T>
where
    T: Into<ParameterName>,
{
    /// Output type, a result or an [`OperationError`].
    type Output;
    /// Converts itself into the output type.
    fn parameter(self, name: T) -> Self::Output;
}

impl<T, S> ToOperationParameterError<S> for Result<T, ParameterError>
where
    S: Into<ParameterName>,
{
    type Output = Result<T, OperationError>;
    fn parameter(self, name: S) -> Result<T, OperationError> {
        self.map_err(|e| e.parameter(name))
    }
}

impl<S> ToOperationParameterError<S> for ParameterError
where
    S: Into<ParameterName>,
{
    type Output = OperationError;
    fn parameter(self, name: S) -> OperationError {
        OperationError::Parameter(name.into(), self)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParameterError::UnexpectedType => write!(f, "Unexpected parameter type."),
            ParameterError::NotSupported => write!(
                f,
                "Parameter value is correct, however it's not supported yet."
            ),
            ParameterError::OutOfRange => write!(f, "Parameter value is out of range"),
        }
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn indent(text: String) -> String {
            let mut indented = "".to_string();
            for line in text.split("\n") {
                if line.is_empty() {
                    break;
                }
                indented += "  ";
                indented += line;
                indented += "\n";
            }
            indented
        }
        match self {
            OperationError::Parameter(name, error) => write!(
                f,
                "Operation parameter {} has the following problem: {}",
                name, error
            ),
            OperationError::TooManyParameters => write!(f, "Operation has too many parameters."),
            OperationError::LogicErrorInParameters(ref error) => write!(f, "Logic error {}", error),
            OperationError::MissingRequiredParameters => {
                write!(f, "Operation requires more parameters.")
            }
            OperationError::MissingBlock => {
                write!(f, "Operation requires block in {{}} braces.")
            }
            OperationError::Nested(error) => write!(f, "\n{}", indent(error.to_string())),
            OperationError::NotFitInSlice => {
                write!(f, "Command bytecode is too long for single slice")
            }
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::Syntax(position, explanation) => {
                write!(f, "{} Syntax error: {}", position, explanation)
            }
            CompileError::UnknownOperation(position, name) => {
                write!(f, "{} Unknown operation {}", position, name)
            }
            CompileError::Operation(position, name, error) => {
                write!(f, "Instruction {} at {}: {}", name, position, error)
            }
        }
    }
}
