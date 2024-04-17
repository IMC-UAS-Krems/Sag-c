use actix_web::{error, web::Json};
use serde::Serialize;
use std::{fmt::Debug, usize};

use crate::parser::Position;

#[derive(Serialize)]
#[serde(untagged)]
pub enum SagError {
    LanguageError(LanguageError),
    ParsingError(ParsingError),
    InternalError(String),
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
}

// new
#[derive(Serialize, Debug)]
pub struct WebErrorPosition {
    pub status: String,
    pub errors: Vec<SagError>,
}

impl ToString for LanguageErrorKind {
    fn to_string(&self) -> String {
        match self {
            LanguageErrorKind::InvalidType() => "Invalid type".to_string(),
            LanguageErrorKind::MissingField(field) => format!("Missing field '{}'", field),
            LanguageErrorKind::InvalidValue(value) => format!("Invalid value '{}'", value),
            LanguageErrorKind::Generic(error) => error.to_string(),
            LanguageErrorKind::MissingSection(section) => format!("Missing section '{}'", section),
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

// impl std::error::Error

impl std::error::Error for ParsingError {}
impl std::error::Error for LanguageError {}
impl std::error::Error for SagError {}

// impl std::fmt::Display

impl core::fmt::Display for SagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SagError::LanguageError(e) => write!(f, "{}", e),
            SagError::ParsingError(e) => write!(f, "{}", e),
            SagError::InternalError(e) => write!(f, "{}", e),
        }
    }
}

// new
impl core::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "({}:{} - {}:{}): {}",
            self.line_start, self.column_start, self.line_end, self.column_end, self.error
        )
    }
}

impl core::fmt::Display for LanguageError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "({}:{} - {}:{}): {}",
            self.line_start, self.column_start, self.line_end, self.column_end, self.error
        )
    }
}

// new
impl core::fmt::Display for WebErrorPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl std::fmt::Debug

impl Debug for SagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SagError::LanguageError(e) => write!(f, "{:?}", e),
            SagError::ParsingError(e) => write!(f, "{:?}", e),
            SagError::InternalError(e) => write!(f, "{:?}", e),
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

// new
impl error::ResponseError for WebErrorPosition {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(Json(self))
    }
}
