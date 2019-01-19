extern crate crypto;

use std::env;
use std::iter::Iterator;
use std::io::Result as IOResult;
use std::io::prelude::*;
use std::fs::read_dir;
use std::fs::File;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use crypto::md5::Md5;
use crypto::digest::Digest;

fn get_recursive_contents (d: &Path) -> Vec<PathBuf> {
    let (dirs, mut files): (Vec<PathBuf>, Vec<PathBuf>) = match read_dir(d) {
        Err(_) => (vec![], vec![]),
        Ok(dirents) => dirents
            .map(Result::ok)
            .filter(Option::is_some)
            .map(|x| x.unwrap().path())
            .partition(|x| x.is_dir()),
    };

    files.append(&mut dirs
                 .iter()
                 .map(|ref p| get_recursive_contents(p.as_path()))
                 .collect::<Vec<Vec<PathBuf>>>()
                 .concat());
    files
}

fn file_md5(f: &Path) -> IOResult<Vec<u8>> {
    let mut freader = File::open(f)?;
    let mut hasher = Md5::new();
    let mut buf = [0u8; 1024*1024];
    loop {
        match freader.read(&mut buf) {
            Err(e) => return Err(e),
            Ok(0) => {
                let mut hash = [0u8; 16];
                hasher.result(&mut hash);
                return Ok(hash.to_vec());
            },
            Ok(n) => {
                hasher.input(&buf[0..n]);
            },
        }
    }
}

fn main() {
    match env::args().nth(1) {
        None => println!("Usage: {} <path>", env::args().nth(0).unwrap()),
        Some(p) => {
            let paths = get_recursive_contents(&Path::new(&p));

            let it_det = paths.iter()
                .map(|n| (n, metadata(n), file_md5(n)))
                .filter(|&(_, ref md, ref hash)| md.is_ok() && hash.is_ok())
                .map(|(n, md, hash)| (n, md.unwrap().len(), hash.unwrap()));

            let mut table: BTreeMap<Vec<u8>, Vec<(&PathBuf, u64)>> = BTreeMap::new();

            for (nam, siz, hash) in it_det {
                let ref mut v = table.entry(hash).or_insert(vec![]);
                v.push((nam, siz));
                v.sort_unstable();
            }

            for (_, val) in table.iter().filter(|&(_, ref v)| v.len() > 1) {
                println!("{} {}", val[0].1,
                         val.iter()
                         .map(|&(ref n, _)| n.to_str().unwrap().replace("\\", "\\\\").replace(" ", "\\ "))
                         .collect::<Vec<String>>().join(" "));
            }
        }
    }
}
