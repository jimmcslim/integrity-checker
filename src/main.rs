extern crate clap;
extern crate ignore;

#[macro_use]
extern crate serde_derive;

extern crate serde_bytes;
extern crate serde_cbor;
extern crate serde_json;

extern crate digest;
extern crate sha2;
extern crate sha3;

extern crate integrity_checker;

use std::collections::BTreeMap;
use std::default::Default;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use digest::Digest;

use integrity_checker::error;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct Database(BTreeMap<PathBuf, Entry>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Entry {
    File(HashSums),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HashSums {
    sha2: HashSum,
    sha3: HashSum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HashSum(#[serde(with = "serde_bytes")] Vec<u8>);

fn parse_args() -> OsString {
    let matches = clap::App::new("Integrity Checker")
        .arg(clap::Arg::with_name("path")
             .help("Path to file or directory to check integrity of")
             .required(true)
             .index(1))
        .get_matches();
    matches.value_of_os("path").unwrap().to_owned()
}

#[derive(Default)]
struct Hashers {
    sha2: sha2::Sha256,
    sha3: sha3::Sha3_256,
}

impl Hashers {
    fn input(&mut self, input: &[u8]) {
        self.sha2.input(input);
        self.sha3.input(input);
    }
    fn result(self) -> HashSums {
        HashSums {
            sha2: HashSum(Vec::from(self.sha2.result().as_slice())),
            sha3: HashSum(Vec::from(self.sha3.result().as_slice())),
        }
    }
}

fn compute_hashes<P>(path: P) -> Result<HashSums, error::Error>
where
    P: AsRef<Path>
{
    let mut f = File::open(path)?;

    let mut hashers = Hashers::default();

    let mut buffer = [0; 4096];
    loop {
        let n = f.read(&mut buffer[..])?;
        if n == 0 { break }
        hashers.input(&buffer[0..n]);
    }
    Ok(hashers.result())
}

fn build_database<P: AsRef<Path>>(path: P) -> Result<Database, error::Error> {
    let mut database = Database::default();
    for entry in WalkBuilder::new(path).build() {
        let entry = entry?;
        if entry.file_type().map_or(false, |t| t.is_file()) {
            let hashes = compute_hashes(entry.path())?;
            let result = Entry::File(hashes);
            database.0.insert(entry.path().to_owned(), result);
        }
    }
    Ok(database)
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (path, entry) in self.0.iter() {
            match entry {
                &Entry::File(ref hashes) => {
                    let hash: Vec<_> = hashes.sha2.0.iter().map(
                        |b| format!("{:02x}", b)).collect();
                    writeln!(f, "{} {}", hash.join(""), Path::new(path).display())?
                    }
            }
        }
        Ok(())
    }
}

fn main() {
    let path = parse_args();
    let database = build_database(&path).unwrap();
    let json = serde_json::to_string(&database).unwrap();
    println!("{}", json);

    let cbor = serde_cbor::to_vec(&database).unwrap();
    let cbor_bytes: Vec<_> = cbor.iter().map(|b| format!("{:02x}", b)).collect();
    println!("{}", cbor_bytes.join(""));

    println!("JSON bytes: {}", json.len());
    println!("CBOR bytes: {}", cbor.len());
}
