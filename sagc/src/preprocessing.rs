use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub enum Either<F, P> {
    _file(F),
    _path(P),
}

pub fn custom_open<P: AsRef<Path>>(input: Either<File, P>) -> io::Result<String> {
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

#[derive(Debug)]
pub struct ImportLog<'a> {
    pub filename: &'a str,
    pub line: i16,
    pub length: i16,
}

#[derive(Debug)]
pub struct ImportLogs<'a> {
    pub imported: Vec<ImportLog<'a>>
}

pub fn substitute_imports(target: File) -> io::Result<String> {
    let reader = BufReader::new(target);
    let mut result = String::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("#import") {
            let import_path = line[7..].trim();
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

fn main(){
    let file = match File::open("sagc/import_test.ssd") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };

    if let Err(e) = substitute_imports(file) {
        eprintln!("Error reading file: {}", e);
    }
}