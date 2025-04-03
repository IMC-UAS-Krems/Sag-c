use actix_web::{error, web::Json};
use serde::Serialize;
use std::fmt::{Debug, Display};

use crate::parser::Position;

#[derive(Serialize)]
#[serde(untagged)]
pub enum SagError {
    LanguageError(LanguageError),
    ParsingError(ParsingError),
    InternalError(String),
    ImportError(ImportError),
}

#[derive(Serialize)]
pub struct ImportError {
    error: String,
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
}

#[derive(Debug, Serialize)]
pub enum ImportErrorKind {
    MissingImport(String),
    NestedImport(String),
    CompilationProblem(String),
    InternalError(String),
    ParsingErrorImport(String),
    BlockNotFound(String, String),
    GeneralError(String),
}

// new
#[derive(Serialize)]
pub struct ParsingError {
    error: String,
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
}

#[derive(Serialize)]
pub struct LanguageError {
    error: String,
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
}

#[derive(Debug, Serialize)]
pub enum LanguageErrorKind {
    InvalidType(),
    MissingField(String),
    InvalidValue(String),
    Generic(String),
    MissingSection(String),
    InvalidPanelType(String),
    IncorrectPanelName(String),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CompileError {
    WebPos(WebErrorPosition),
    General(GeneralError),
}

#[derive(Debug, Serialize)]
pub struct GeneralError {
    status: String,
    error: String,
}

// new
#[derive(Serialize, Debug)]
pub struct WebErrorPosition {
    pub status: String,
    pub errors: Vec<SagError>,
}

impl Display for LanguageErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LanguageErrorKind::InvalidType() => write!(f, "Invalid type"),
            LanguageErrorKind::MissingField(field) => write!(f, "Missing field '{}'", field),
            LanguageErrorKind::InvalidValue(value) => write!(f, "Invalid value '{}'", value),
            LanguageErrorKind::Generic(error) => write!(f, "{}", error),
            LanguageErrorKind::IncorrectPanelName(name) => {
                write!(f, "Undefined or misspelled panel variable '{}'", name)
            }
            LanguageErrorKind::MissingSection(section) => {
                write!(f, "Missing section '{}'", section)
            }
            LanguageErrorKind::InvalidPanelType(panel) => {
                write!(f, "Unsupported panel type '{}'", panel)
            }
        }
    }
}

impl Display for ImportErrorKind {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImportErrorKind::MissingImport(file) => write!(f, "Could not access file '{}'", file),
            ImportErrorKind::NestedImport(file) => write!(f, "Imported file '{}' contains nested imports.",file),
            ImportErrorKind::CompilationProblem(file) => write!(f, "File '{}' failed in compilation.",file),
            ImportErrorKind::InternalError(file) => write!(f, "Internal error in file '{}'",file),
            ImportErrorKind::ParsingErrorImport(file) => write!(f, "Parsing errors in file {}", file),
            ImportErrorKind::BlockNotFound(block, file) => write!(f, "{} not found in file '{}'", block, file),
            ImportErrorKind::GeneralError(error) => write!(f, "{}", error),
        }
    }
}

impl SagError {
    pub fn unparsable(position: Position) -> Self {
        SagError::ParsingError(ParsingError::unparsable(position))
    }
    pub fn invalid_indentation(position: Position) -> Self {
        SagError::ParsingError(ParsingError::invalid_indentation(position))
    }
    pub fn language_error(error_kind: LanguageErrorKind, pos: Position) -> Self {
        SagError::LanguageError(LanguageError::new(error_kind, pos))
    }

    pub fn internal_error(error: String) -> Self {
        SagError::InternalError(error)
    }
    
    pub fn import_error(error_kind: ImportErrorKind, pos: Position) -> Self {
        SagError::ImportError(ImportError::new(error_kind, pos))
    }
}

// new
impl ParsingError {
    fn unparsable(position: Position) -> Self {
        ParsingError {
            error: "Unparsable".to_string(),
            line_start: position.row_start,
            column_start: position.col_start,
            line_end: position.row_end,
            column_end: position.col_end,
        }
    }

    fn invalid_indentation(position: Position) -> Self {
        ParsingError {
            error: "Invalid indentation".to_string(),
            line_start: position.row_start,
            column_start: position.col_start,
            line_end: position.row_end,
            column_end: position.col_end,
        }
    }
}

impl LanguageError {
    fn new(error_kind: LanguageErrorKind, pos: Position) -> Self {
        LanguageError {
            error: error_kind.to_string(),
            line_start: pos.row_start,
            column_start: pos.col_start,
            line_end: pos.row_end,
            column_end: pos.col_end,
        }
    }
}

impl ImportError {
    fn new(error_kind: ImportErrorKind, pos: Position) -> Self {
        ImportError {
            error: error_kind.to_string(),
            line_start: pos.row_start,
            column_start: pos.col_start,
            line_end: pos.row_end,
            column_end: pos.col_end,
        }
    }
}

impl GeneralError {
    pub fn new(error: String) -> Self {
        GeneralError {
            status: "error".to_string(),
            error,
        }
    }
}

// impl std::error::Error

impl std::error::Error for ParsingError {}
impl std::error::Error for LanguageError {}
impl std::error::Error for SagError {}
impl std::error::Error for ImportError {}

// impl std::fmt::Display

impl Display for SagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SagError::LanguageError(e) => write!(f, "{}", e),
            SagError::ParsingError(e) => write!(f, "{}", e),
            SagError::InternalError(e) => write!(f, "{}", e),
            SagError::ImportError(e) => write!(f, "{}", e),
        }
    }
}

// new
impl Display for ParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "({}:{} - {}:{}): {}",
            self.line_start, self.column_start, self.line_end, self.column_end, self.error
        )
    }
}

impl Display for LanguageError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "({}:{} - {}:{}): {}",
            self.line_start, self.column_start, self.line_end, self.column_end, self.error
        )
    }
}

impl Display for ImportError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "({}:{} - {}:{}): {}",
            self.line_start, self.column_start, self.line_end, self.column_end, self.error
        )
    }
}

// new
impl Display for WebErrorPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompileError::WebPos(e) => write!(f, "{}", e),
            CompileError::General(e) => write!(f, "{}", e),
        }
    }
}

// impl std::fmt::Debug

impl Debug for SagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SagError::LanguageError(e) => write!(f, "{:?}", e),
            SagError::ParsingError(e) => write!(f, "{:?}", e),
            SagError::InternalError(e) => write!(f, "{:?}", e),
            SagError::ImportError(e) => write!(f, "{:?}", e),
        }
    }
}

impl Debug for LanguageError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

// new
impl Debug for ParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Debug for ImportError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

// new
impl error::ResponseError for WebErrorPosition {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(Json(self))
    }
}

impl error::ResponseError for GeneralError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(Json(self))
    }
}

impl error::ResponseError for CompileError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(Json(self))
    }
}
