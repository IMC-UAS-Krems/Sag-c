use actix_web::{error, web::Json};
use serde::Serialize;
use std::fmt::Debug;

pub enum SagError {
    ParsingError(ParsingError),
    InvalidChar(InvalidChar),
    Error(String),
}
pub struct InvalidChar {
    pub line: usize,
    pub column: usize,
}

pub struct ParsingError {
    error: ParsingErrorKind,
    section: String,
    field: String,
}

#[derive(Debug)]
enum ParsingErrorKind {
    MissingField(String),
    MissingSection(String),
    Error(String),
}

#[derive(Serialize)]
struct WebError {
    error: String,
}

impl SagError {
    pub fn parsing_error<T: ToString>(section: &str, field: &str, error: T) -> Self {
        SagError::ParsingError(ParsingError::new(section, field, error))
    }
    pub fn missing_field(section: &str, field: &str) -> Self {
        SagError::ParsingError(ParsingError::missing_field(section, field))
    }
    pub fn missing_section(section: &str) -> Self {
        SagError::ParsingError(ParsingError::missing_section(section))
    }
    pub fn invalid_char(line: usize, column: usize) -> Self {
        SagError::InvalidChar(InvalidChar { line, column })
    }
    pub fn error<T: ToString>(error: T) -> Self {
        SagError::Error(error.to_string())
    }
}

impl ParsingError {
    fn new<T: ToString>(section: &str, field: &str, error: T) -> Self {
        ParsingError {
            error: ParsingErrorKind::Error(error.to_string()),
            section: section.to_string(),
            field: field.to_string(),
        }
    }
    fn missing_field(section: &str, field: &str) -> Self {
        ParsingError {
            error: ParsingErrorKind::MissingField(field.to_string()),
            section: section.to_string(),
            field: field.to_string(),
        }
    }
    fn missing_section(section: &str) -> Self {
        ParsingError {
            error: ParsingErrorKind::MissingSection(section.to_string()),
            section: section.to_string(),
            field: "".to_string(),
        }
    }
}

// impl std::error::Error

impl std::error::Error for InvalidChar {}
impl std::error::Error for ParsingError {}
impl std::error::Error for SagError {}

// impl std::fmt::Display

impl core::fmt::Display for SagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SagError::ParsingError(e) => write!(f, "{}", e),
            SagError::InvalidChar(e) => write!(f, "{}", e),
            SagError::Error(e) => write!(f, "{}", e),
        }
    }
}

impl core::fmt::Display for InvalidChar {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "Uknown character at ({}, {})",
            self.line + 1,
            self.column + 1,
        )
    }
}

impl core::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self.error {
            ParsingErrorKind::MissingField(field) => {
                write!(f, "Missing field '{}' in section '{}'", field, self.section)
            }
            ParsingErrorKind::MissingSection(section) => {
                write!(f, "Missing section '{}'", section)
            }
            ParsingErrorKind::Error(error) => {
                write!(f, "{}.{}: {}", self.section, self.field, error)
            }
        }
    }
}

// impl std::fmt::Debug

impl Debug for SagError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SagError::ParsingError(e) => write!(f, "{:?}", e),
            SagError::InvalidChar(e) => write!(f, "{:?}", e),
            SagError::Error(e) => write!(f, "{:?}", e),
        }
    }
}

impl Debug for ParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self.error {
            ParsingErrorKind::MissingField(field) => {
                write!(f, "Missing field {} in section {}", field, self.section)
            }
            ParsingErrorKind::MissingSection(section) => {
                write!(f, "Missing section {}", section)
            }
            ParsingErrorKind::Error(error) => {
                write!(f, "{}.{}: {}", self.section, self.field, error)
            }
        }
    }
}

impl Debug for InvalidChar {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "Uknown character at ({}, {})",
            self.line + 1,
            self.column + 1,
        )
    }
}

// impl actix_web::error::ResponseError

impl error::ResponseError for SagError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            SagError::ParsingError(e) => {
                actix_web::HttpResponse::BadRequest().json(Json(WebError {
                    error: e.to_string(),
                }))
            }
            SagError::InvalidChar(e) => {
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
