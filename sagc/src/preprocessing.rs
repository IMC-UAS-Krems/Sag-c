/* This file defines functions and structures for handling file imports and substitutions in a configuration system */

use std::fs::{File, read_to_string};
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::path::Path;
use actix_web::{web, Error as ActixError, error::ErrorInternalServerError};
use awc::Client;
use crate::parser::{Span, Position};
use crate::errors::{ImportError, ImportErrorKind, SagError};
use serde::{Deserialize, Serialize};

// ------ new imports from include snippets ----------
use dotenv::dotenv;
use std::env::var;

/* -----------Section 1: Defining Structs and Enums----------- */

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
pub struct ImportFileContent<'a> {
    pub name: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContentRequest {
    pub municipalityName: String,
    #[serde(rename(serialize = "organizationName"))]
    pub orgName: String,
    pub projectName: String,
    pub path: String,
}

/* -----------Section 2: Defining Functions----------- */

/// searches for a file by name in a list of import files contents and returns the content if found
pub fn search_files(name: &str, pos: Position, files: &[ImportFileContent]) -> Result<String, SagError> {
    for f in files {
        if name == f.name {
            println!("{:?}", f);
            return Ok(f.content.to_string());
        }
    }
    Err(SagError::import_error(ImportErrorKind::MissingImport(name.to_string()),pos))
}

/// checks the content of an imported file for nested imports and logs the details, returns ImportLog if successful
pub fn check_import(content: &str, filename: String, pos_in_file: usize, pos: Position) -> Result<ImportLog, Vec<SagError>>  {
    let mut nlines: usize = 0;
    let mut errors = Vec::new();

    for line in content.lines() {
        if line.starts_with("#import") {
            errors.push(SagError::import_error(ImportErrorKind::NestedImport(filename.to_string()),pos))
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

/// sends a request to the backend and returns the conents of the file #import <PATH_TO_FILE>
pub async fn get_content_from_backend(client: web::Data<Client>) -> Result<String, ActixError> {
    
    // TODO: this default for now; change to more specified request 
    let data = ContentRequest {
        municipalityName: "Krems".into(),
        orgName: "Imc".into(),
        projectName: "Project 1".into(),
        path: "folder-1.file-1".into(),
    };

    dotenv::dotenv().ok(); // Load environment variables
    let token = var("WEB_TOKEN").map_err(|e| ErrorInternalServerError(e))?;

    let req_url = "http://localhost:9512/api/document_content";
    let mut req = client.get(req_url).bearer_auth(&token);
    
    req = req.query(&data).map_err(|e| ErrorInternalServerError(e))?;

    let mut res = req.send().await.map_err(|e| ErrorInternalServerError(e))?;
    
    let body_bytes = res.body().await.map_err(|e| ErrorInternalServerError(e))?;
    let body_string = String::from_utf8(body_bytes.to_vec()).map_err(|e| ErrorInternalServerError(e))?;
    

    Ok(body_string)
}

/// substitutes import statements in the target content with the actual content from the backend.
pub async fn substitute_imports_from_backend(target: &str) -> Result<String, Vec<Error>> {
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
            let client = web::Data::new(Client::default());
            let import_pos = Position { 
                row_start: nlines, 
                row_end: nlines, 
                col_start: 0, 
                col_end: line.len() 
            };

            let import_name = line[7..].trim();

            let import_content = get_content_from_backend(client).await;
            
            match import_content {
                Ok(content) => {
                    match check_import(&content, import_name.to_string(), nlines, import_pos) {
                        Ok(response) => result.push_str(&format!("{}\n", content)),  // if no errors, append the content to the result
                        Err(mut e) => (), // TODO: push the right error
                    }
                }
                Err(e) => (),  // TODO:  push the right error
            }
        } else {
            result.push_str(&format!("{}\n", line));
        }
    }

    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors)  // TODO: return the right error
    }
}
/// substitutes import statements in the target content with the actual content of the imported files.
pub fn substitute_imports(target: &str, imports: &[ImportFileContent]) -> Result<String, Vec<SagError>> {
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

            let import_content = search_files(import_name.clone(), import_pos, imports);
            
            match import_content {
                Ok(content) => {
                    match check_import(&content, import_name.to_string(), nlines, import_pos) {
                        Ok(response) => result.push_str(&format!("{}\n", content)),  // if no errors, append the content to the result
                        Err(mut e) => errors.append(&mut e),
                    }
                }
                Err(e) => errors.push(e),
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


pub fn postprocess_errors() {
    
    /*  TODO: 
        1. parse through errors and logs
        2. if error is located in an imported file, change the error message
    */
}

fn main(){
    let mut files: Vec<ImportFileContent> = Vec::new();
    files.push(ImportFileContent{name:"a", content:"aa"});
    files.push(ImportFileContent{name:"b", content:"bbb"});
    println!("{:?}", files);

    let file = read_to_string("sagc/import_test.ssd");
    match file {
        Ok(content) => {
            println!("{:?}", content);
            println!("{:?}", substitute_imports(&content, &files)); // Pass content as &str
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
}


/*
fn main(){
    let file = match File::open("sagc/import_test.ssd") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };

    let files: Vec<ImportFile> = Vec::new();

    if let Err(e) = substitute_imports(file, files) {
        eprintln!("Error reading file: {}", e);
    }

    let file2 = match File::open("sagc/dash_example.ssd") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };

    let f:ImportFile = ImportFile{name:"test", content:file2};
    check_import(f, 12);
}
*/

/* TODO:s

1. make substitution function take vector of file contents
2. check for the content size and properly log it using the structures
3. import span and return the contents as span, not as string
4. ??? deal with recursive substitutes (log what was already imported and chceck before continuing)

*/