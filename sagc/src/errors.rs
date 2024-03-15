use actix_web::{error, web::Json};
use serde::Serialize;
use std::fmt::Debug;

use crate::parser::Position;

#[derive(Serialize)]
#[serde(untagged)]
pub enum SagError {
    LanguageError(LanguageError),
    ParsingError(ParsingError),
    Error(String),
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
    error: LanguageErrorKind,
    section: String,
    field: Option<String>,
}

#[derive(Debug, Serialize)]
enum LanguageErrorKind {
    MissingField(String),
    MissingSection(String),
    Error(String),
}

#[derive(Serialize)]
struct WebError {
    error: String,
}

// new
#[derive(Serialize, Debug)]
pub struct WebErrorPosition {
    pub status: String,
    pub errors: Vec<SagError>,
}

impl SagError {
    pub fn language_error<T: ToString>(section: &str, field: Option<&str>, error: T) -> Self {
        SagError::LanguageError(LanguageError::new(section, field, error))
    }
    pub fn missing_field(section: &str, field: &str) -> Self {
        SagError::LanguageError(LanguageError::missing_field(section, field))
    }
    pub fn missing_section(section: &str) -> Self {
        SagError::LanguageError(LanguageError::missing_section(section))
    }
    pub fn unparsable(position: Position) -> Self {
        SagError::ParsingError(ParsingError::unparsable(position))
    }
    pub fn invalid_indentation(position: Position) -> Self {
        SagError::ParsingError(ParsingError::invalid_indentation(position))
    }
    pub fn error<T: ToString>(error: T) -> Self {
        SagError::Error(error.to_string())
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
    fn new<T: ToString>(section: &str, field: Option<&str>, error: T) -> Self {
        LanguageError {
            error: LanguageErrorKind::Error(error.to_string()),
            section: section.to_string(),
            field: field.map(|s| s.to_string()),
        }
    }
    fn missing_field(section: &str, field: &str) -> Self {
        LanguageError {
            error: LanguageErrorKind::MissingField(field.to_string()),
            section: section.to_string(),
            field: Some(field.to_string()),
        }
    }
    fn missing_section(section: &str) -> Self {
        LanguageError {
            error: LanguageErrorKind::MissingSection(section.to_string()),
            section: section.to_string(),
            field: None,
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
            SagError::Error(e) => write!(f, "{}", e),
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
        match &self.error {
            LanguageErrorKind::MissingField(field) => {
                write!(f, "Missing field '{}' in section '{}'", field, self.section)
            }
            LanguageErrorKind::MissingSection(section) => {
                write!(f, "Missing section '{}'", section)
            }
            LanguageErrorKind::Error(error) => match &self.field {
                Some(field) => write!(f, "{}.{}: {}", self.section, field, error),
                None => write!(f, "{}: {}", self.section, error),
            },
        }
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
            SagError::Error(e) => write!(f, "{:?}", e),
        }
    }
}

impl Debug for LanguageError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self.error {
            LanguageErrorKind::MissingField(field) => {
                write!(f, "Missing field {} in section {}", field, self.section)
            }
            LanguageErrorKind::MissingSection(section) => {
                write!(f, "Missing section {}", section)
            }
            LanguageErrorKind::Error(error) => match &self.field {
                Some(field) => write!(f, "{}.{}: {}", self.section, field, error),
                None => write!(f, "{}: {}", self.section, error),
            },
        }
    }
}

// new
impl Debug for ParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "({}:{} - {}:{}): {}",
            self.line_start, self.column_start, self.line_end, self.column_end, self.error
        )
    }
}

// impl actix_web::error::ResponseError

impl error::ResponseError for SagError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            SagError::LanguageError(e) => {
                actix_web::HttpResponse::BadRequest().json(Json(WebError {
                    error: e.to_string(),
                }))
            }
            SagError::ParsingError(e) => {
                actix_web::HttpResponse::BadRequest().json(Json(WebError {
                    error: e.to_string(),
                }))
            }
            SagError::Error(e) => actix_web::HttpResponse::BadRequest().json(Json(WebError {
                error: e.to_string(),
            })),
        }
    }
}

// new
impl error::ResponseError for WebErrorPosition {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(Json(self))
    }
}
