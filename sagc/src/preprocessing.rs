use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use crate::parser::Span;
use crate::errors::ImportError;

pub enum Either<F, P> {
    _file(F),
    _path(P),
}

pub fn custom_open<P: AsRef<Path>>(input: Either<File, P>) -> io::Result<String> {
    //helper for opening irregardles of type
    let file = match input {
        Either::_file(file) => file,
        Either::_path(path) => File::open(path)?
    };
    
    let reader = BufReader::new(file);
    let mut result = String::new();
    
    for line in reader.lines() {
        result.push_str(&format!("{}\n", line?));
    }
    
    Ok(result)
}

pub fn check_import(content: ImportFile, position: i16) -> io::Result<ImportLog> {
    //TODO fix typing
    let reader = BufReader::new(content.content);
    let mut nlines: i16 = 0;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("#import") {
            //TODO change handling
            panic!("Import file contains an import statement");
            Err::<ImportLog,&str>("Imported file contains further imports"); //alternatively recurse?
        } else {
            nlines += 1;
        }
    }

    let log: ImportLog = ImportLog{filename: content.name, line: position, length: nlines};
    println!{"filename {}", content.name};
    println!{"line {}", position};
    println!{"len {}", nlines};
    Ok(log)
} 

#[derive(Debug)]
pub struct ImportLog<'a> {
    pub filename: &'a str,
    pub line: i16,
    pub length: i16,
}

#[derive(Debug)]
pub struct ImportLogs<'a> {
    pub imported: Vec<ImportLog<'a>>,
}

#[derive(Debug)]
pub struct ImportFile<'a> {
    pub name: &'a str,
    pub content: File,
}

pub fn substitute_imports(target: &str, imports: Vec<ImportFile>) -> io::Result<String> {
    let mut result = String::new();
    
    for line in target.lines() {
        if line.starts_with("#import") {
            let import_path = line[7..].trim(); //TODO use 'check_import'
            match custom_open(Either::_path(import_path.to_string())) {
                Ok(imported_content) => result.push_str(&imported_content),
                Err(e) => eprintln!("Error importing {}: {}", import_path, e),
            }
        } else {
            result.push_str(&format!("{}\n", line));
        }
    }
    
    println!("{}", result);
    Ok(result)
}

pub fn postprocess_errors() {
    // parse through errors and logs
    // if error is located in an imported file, change the error message
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
/*TODOs

1. make substitution function take vector of file contents
2. check for the content size and properly log it using the structures
3. import span and return the contents as span, not as string
4. ??? deal with recursive substitutes (log what was already imported and chceck before continuing)

*/