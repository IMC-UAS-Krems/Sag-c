use std::fs::{File, read_to_string};
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
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

#[derive(Debug)]
pub struct ImportFileContent<'a> {
    pub name: &'a str,
    pub content: &'a str,
}

pub fn search_files(name: &str, files: &[ImportFileContent]) -> io::Result<String> {
    for f in files {
        if name == f.name {
            println!("{:?}", f);
            return Ok(f.content.to_string());
        }
    }
    Err(Error::new(ErrorKind::NotFound, format!("File '{}' not found", name)))
}

pub fn substitute_imports(target: &str, imports: &[ImportFileContent]) -> io::Result<String> {
    let mut result = String::new();
    
    for line in target.lines() {
        if line.starts_with("#import") {
            let import_name = line[7..].trim(); // TODO use 'check_import'
            let import_content = search_files(import_name, &imports); // Pass reference
            match import_content {
                Ok(content) => result.push_str(&format!("{}\n",content)),
                Err(e) => eprintln!("Error importing {}: {}", import_name, e),
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

fn main(){
    let mut files: Vec<ImportFileContent> = Vec::new();
    files.push(ImportFileContent{name:"a", content:"aa"});
    files.push(ImportFileContent{name:"b", content:"bbb"});
    println!("{:?}", files);

    // Test search_files
    let test = "a";
    println!("{:?}", search_files(test, &files)); // Pass reference

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
/*TODOs

1. make substitution function take vector of file contents
2. check for the content size and properly log it using the structures
3. import span and return the contents as span, not as string
4. ??? deal with recursive substitutes (log what was already imported and chceck before continuing)

*/