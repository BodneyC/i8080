use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

pub fn rsc<P: AsRef<Path>>(filename: P) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("rsc");
    d.push(filename);
    d
}

pub fn read_to_v8<P: AsRef<Path>>(filename: P) -> Result<Vec<u8>, io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;
    Ok(buffer)
}

pub fn read_to_v_string<P: AsRef<Path>>(filename: P) -> Result<Vec<String>, io::Error> {
    let file = File::open(filename)?;
    let buf = BufReader::new(file);
    Ok(buf
        .lines()
        .map(|l| l.expect("could not parse line"))
        .collect())
}
