/*
    This file handles import statements.
    parse_import_statement: substitutes import statements in the target content with the actual content from the backend.
*/

/************************************************************************************************/

use std::fs::File;
use std::io::BufRead;
use std::env::var;
use actix_web::{web, Error as ActixError, error::ErrorInternalServerError, http::StatusCode};
use awc::Client;
use crate::parser::{Position, parse_lines};
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
pub async fn parse_import_statement(target: &str, metadata: FileMetadata) -> Result<String, Vec<SagError>> {
    /* 
    If import is found in the target content:
        either substitute the full content of the imported file (import <file_name>)
        or substitute some of the imported file (import <file_name>: <block_names>)
    */
    
    let mut result = String::new();
    let mut errors = Vec::new(); 
    let mut nlines: usize = 0;
    let mut lines = target.lines();

    // iterate through the lines of the target content
    while let Some(line) = lines.next() {
        nlines += 1;

        // if import statement is found
        if line.starts_with("import") {

            // get the position of the import statement
            let import_pos = Position { 
                row_start: nlines, 
                row_end: nlines, 
                col_start: 0, 
                col_end: line.len() 
            };

            // remove import from the import statement
            let import_statement = line[6..].trim(); // remove "import" and trim whitesapce
            let (import_filename, block_names): (&str, Option<Vec<&str>>) = if import_statement.contains(':') {
                nlines += 1;

                // If the import statement contains a colon, split into filename and block names
                let filename = import_statement.trim_end_matches(':');

                if let Some(next_line) = lines.next() {
                    let blocks = next_line.split(',').map(str::trim).collect();
                    (filename, Some(blocks))
                } else {
                    (filename, None)  // TODO: handle error error
                }
            } else {
                // Otherwise, check the next line for block names
                let filename = import_statement;
                (filename, None)
            };

            let imported_file_metadata = get_imported_file_metadata(metadata.clone(), import_filename);

            let import_content = match block_names {
                Some(blocks) => substitute_some_import_content(import_filename, blocks, imported_file_metadata, import_pos).await,
                None => {
                    get_all_imported_content(imported_file_metadata, import_pos).await
                }
            };

            match import_content {
                Ok(content) => {
                    match check_import_errors(&content, import_filename.to_string(), nlines, import_pos) {
                        Ok(_response) => result.push_str(&format!("{}\n", content)),  // if no errors, append the content to the result
                        Err(mut e) => errors.append(&mut e), // nested import error
                    };
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

/// sends a request to the backend and returns the conents of the file import
pub fn get_imported_file_metadata(origin_metadata: FileMetadata, imported_path: &str) -> FileMetadata {

    let imported_file_path = FileMetadata {
        municipalityName: origin_metadata.municipalityName,
        orgName: origin_metadata.orgName,
        projectName: origin_metadata.projectName,
        path: imported_path.into(),
    };

    return imported_file_path

}

pub async fn substitute_some_import_content(filename: &str, block_names: Vec<&str>, imported_file_metadata: FileMetadata, position: Position) -> Result<String, Vec<SagError>> {
    /* subtitute some of the imported file */

    let mut result = String::new();
    let mut errors = Vec::new();

    let all_import_content = get_all_imported_content(imported_file_metadata, position).await; // get the content of the imported file

    match all_import_content {
        Ok(content) => {
            
            for block in &block_names {
                let mut lines = content.lines();
                let mut block_found = false;

                while let Some(line) = lines.next() {
                    if line.starts_with(block) {
                        block_found = true;
                        result.push_str(&format!("{}\n", line));
        
                        while let Some(next_line) = lines.next() {
                            if next_line.starts_with(" ") || next_line.starts_with("\t") {
                                result.push_str(&format!("{}\n", next_line));
                            } else {
                                result.push_str(&format!("\n"));
                                break;  // break the loop if the line is not indented
                            }
                        }
                    }
                }
                if !block_found {
                    errors.push(SagError::import_error(ImportErrorKind::BlockNotFound(block.to_string(), filename.to_string()), position));
                }
            }
        }
        Err(mut e) => {
            errors.append(&mut e);
        }
    }

    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors)
    }
}   

/// sends a request to the backend and returns the conents of the file #import <PATH_TO_FILE>
pub async fn get_all_imported_content(data: FileMetadata, position: Position) -> Result<String, Vec<SagError>> {
    
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
                ImportErrorKind::MissingImport(data.path.to_string()),
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

/// checks the imported file for nested imports, returns ImportLog if successful
pub fn check_import_errors(content: &str, filename: String, pos_in_file: usize, pos: Position) -> Result<ImportLog, Vec<SagError>>  {
    let mut nlines: usize = 0;
    let mut errors = Vec::new();

    for line in content.lines() {
        if line.starts_with("import") {
            let error_string = filename.to_string();
            errors.push(SagError::import_error(ImportErrorKind::NestedImport(error_string),pos))
        } else {
            nlines += 1;
        }
    }

    if errors.is_empty() {
        let result = parse_lines(content);

        match result {
            Ok(_) => (),
            Err(e) => {
                let error_string = filename.to_string();
                errors.push(SagError::import_error(ImportErrorKind::ParsingErrorImport(error_string), pos));
            }
        };
    };

    if errors.is_empty() {
        let log: ImportLog = ImportLog{filename: filename, line: pos_in_file, length: nlines};
        Ok(log)
    } else {
        Err(errors)
    }

} 


