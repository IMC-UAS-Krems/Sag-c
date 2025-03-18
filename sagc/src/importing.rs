/*
    This file is to handle import statements
        1. substitute_import_content: substitute the imported content in the main file
            1. check_nested_imports: check for nested imports
            2. get_import_metadata: get the path of the imported file
            3. get_import_content: get the content of the imported file
            4. check_import_errors: check for parsing errors in the imported file

*/

/************************************************************************************************/

use std::fs::File;
use std::io::BufRead;
use std::env::var;
use actix_web::{web, Error as ActixError, error::ErrorInternalServerError, http::StatusCode};
use awc::Client;
use crate::parser::Position;
use crate::errors::{ImportErrorKind, SagError};
use serde::{Deserialize, Serialize};

/*****************************Section 1: Defining Structs and Enums*****************************/

/// represents log details of the imported file
#[derive(Debug)]
pub struct ImportLog {
    pub filename: String,
    pub line: usize,
    pub length: usize,
}

/// represents a collection of import logs
#[derive(Debug)]
pub struct ImportLogs {
    pub imported: Vec<ImportLog>,
}

/// represents an import file with its name and content
#[derive(Debug)]
pub struct ImportFile<'a> {
    pub name: &'a str,
    pub content: File,
}

/// represents the content of an imported file
#[derive(Debug)]
pub struct ImportFileContent {
    pub name: String,
    pub content: String,
}

/// represents the content request to send to the backend
#[derive(Debug, Deserialize, Serialize)]
pub struct ContentRequest {
    pub municipalityName: String,
    #[serde(rename(serialize = "organizationName"))]
    pub orgName: String,
    pub projectName: String,
    pub path: String,
}

/// represents the metadata of a file
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileMetadata {
    pub municipalityName: String,
    #[serde(rename(serialize = "organizationName"))]  // orgName -> organizationName
    pub orgName: String,
    pub projectName: String,
    pub path: String,
}

/*****************************Section 2: Defining Functions*****************************/

/// substitutes import statements in the target content with the actual content from the backend.
pub async fn substitute_import_content(target: &str, metadata: FileMetadata) -> Result<String, Vec<SagError>> {
    /* 
    If #import is found in the target content:
        1. Send a request to the backend to get the contents of the file #import <PATH_TO_FILE>
        2. If we successfully get the content:
            Substitue it
        3. If the fetch fails:
            Push an error into the error list
    */

    let mut result = String::new();
    let mut errors = Vec::new(); 
    let mut nlines: usize = 0;

    for line in target.lines() {
        nlines += 1;
        if line.starts_with("#import") {
            let import_pos = Position { 
                row_start: nlines, 
                row_end: nlines, 
                col_start: 0, 
                col_end: line.len() 
            };

            let import_name = line[7..].trim();

            let metadata = get_import_metadata(metadata.clone(), import_name);

            let import_content = get_import_content(metadata, import_pos).await;
            
            match import_content {
                Ok(content) => {
                    match check_nested_imports(&content, import_name.to_string(), nlines, import_pos) {
                        Ok(_response) => result.push_str(&format!("{}\n", content)),  // if no errors, append the content to the result
                        Err(mut e) => errors.append(&mut e), // nested import error
                    }
                }
                Err(mut e) => errors.append(&mut e)
            }
        } else {
            result.push_str(&format!("{}\n", line));
        }
    }

    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors)
    }
}

/// checks the imported file for nested imports, returns ImportLog if successful
pub fn check_nested_imports(content: &str, filename: String, pos_in_file: usize, pos: Position) -> Result<ImportLog, Vec<SagError>>  {
    let mut nlines: usize = 0;
    let mut errors = Vec::new();

    for line in content.lines() {
        if line.starts_with("#import") {
            let error_string = filename.to_string() + " contains nested imports.";
            errors.push(SagError::import_error(ImportErrorKind::NestedImport(error_string),pos))
        } else {
            nlines += 1;
        }
    }

    if errors.is_empty() {
        let log: ImportLog = ImportLog{filename: filename, line: pos_in_file, length: nlines};
        Ok(log)
    } else {
        Err(errors)
    }

} 

pub fn get_import_metadata(origin_metadata: FileMetadata, imported_path: &str) -> FileMetadata {

    let imported_file_path = FileMetadata {
        municipalityName: origin_metadata.municipalityName,
        orgName: origin_metadata.orgName,
        projectName: origin_metadata.projectName,
        path: imported_path.into(),
    };

    return imported_file_path

}

/// sends a request to the backend and returns the conents of the file #import <PATH_TO_FILE>
pub async fn get_import_content(data: FileMetadata, position: Position) -> Result<String, Vec<SagError>> {
    
    dotenv::dotenv().ok(); // Load environment variables
    let mut content = String::new();
    let mut errors = Vec::new();
    let token = match var("WEB_TOKEN").map_err(|e| SagError::import_error(ImportErrorKind::InternalError(e.to_string()), position)) {
        Ok(t) => t,
        Err(e) => {
            errors.push(e);
            return Err(errors);
        }
    };

    let client = web::Data::new(Client::default());

    let req_url = "http://localhost:9512/api/document_content";
    let mut req = match client.get(req_url).bearer_auth(&token).query(&data).map_err(|e| SagError::import_error(ImportErrorKind::InternalError(e.to_string()), position)) {
        Ok(r) => r,
        Err(e) => {
            errors.push(e);
            return Err(errors);
        }
    };
    
    let mut res = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            errors.push(SagError::import_error(ImportErrorKind::InternalError(e.to_string()), position));
            return Err(errors);
        }
    };

    match res.status().as_u16() {
        200 => {
            let body_bytes = match res.body().await {
                Ok(b) => b,
                Err(e) => {
                    errors.push(SagError::import_error(
                        ImportErrorKind::InternalError(format!("Failed to read response body: {}", e)),
                        position,
                    ));
                    return Err(errors);
                }
            };

            content = match String::from_utf8(body_bytes.to_vec()) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(SagError::import_error(
                        ImportErrorKind::InternalError(format!("Invalid UTF-8 in response body: {}", e)),
                        position,
                    ));
                    return Err(errors);
                }
            };
        }
        400 => {
            errors.push(SagError::import_error(
                ImportErrorKind::MissingImport("File not found".to_string()),
                position,
            ));
            return Err(errors);
        }
        _ => {
            errors.push(SagError::import_error(
                ImportErrorKind::InternalError(format!("Unexpected response status: {}", res.status())),
                position,
            ));
            return Err(errors);
        }   
    }
    
    if errors.is_empty() {
        Ok(content)
    } else {
        Err(errors)
    }
}

pub fn check_import_errors() {
    
    /*  TODO: 
        1. parse through errors and logs
        2. if error is located in an imported file, change the error message
    */
}


