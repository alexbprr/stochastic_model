use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::io::Write;
use std::io::BufReader;
use anyhow::Error;

pub fn to_disk<P: AsRef<Path>>(path: P, data: Vec<String>) -> anyhow::Result<(),Error> {
    let mut file: File = match File::create(path) {
        Ok(f) => f,
        Err(e) => return Err(e.into()),
    };
    for v in &data{                                                                                                                                                      
        write!(file, "{ }", v)?;                                                                                                                             
    }
    Ok(())
}

pub fn from_disk<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<String>,Error> {
    let file: File = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e.into()),
    };
    let mut data: Vec<String> = vec![];
    let reader: BufReader<File> = BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        data.push(line);
    }
    Ok(data)
}