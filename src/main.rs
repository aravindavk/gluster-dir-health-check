use std::path::Path;
use std::io::Error;
use std::env;

extern crate walkdir;
extern crate xattr;
extern crate uuid;

use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use uuid::Uuid;

fn filter_dirs(en: &DirEntry) -> bool {
    // To ignore internal directories
    en.file_name()
        .to_str()
        .map(|s| s.starts_with(".glusterfs") || s.starts_with(".trashcan") || s.starts_with(".shard"))
        .unwrap_or(false)
}

fn get_gfid(path: &str) -> Result<String, Error> {
    // Gets trusted.gfid xattr from the given path, Returns
    // empty string if UUID parsing fails
    let v = try!(xattr::get(path, "trusted.gfid"));
    let uuid = Uuid::from_bytes(&v);
    match uuid {
        Ok(v) => Ok(v.hyphenated().to_string()),
        Err(_) => Ok("".to_string()),
    }
}

// All output combinations
enum Reasons {
    Ok,
    NoGfid,
    NoPgfid,
    NoSymlink,
    WrongSymlink,
    InvalidGfid,
    InvalidPGfid,
}

fn output_display(reason: Reasons, entry: &DirEntry, gfid: &str, pgfid: &str) {
    let ok_msg = match reason {
        Reasons::Ok => "[    OK]              ",
        Reasons::NoGfid => "[NOT OK] NO GFID      ",
        Reasons::NoPgfid => "[NOT OK] NO PGFID     ",
        Reasons::NoSymlink => "[NOT OK] NO SYMLINK   ",
        Reasons::WrongSymlink => "[NOT OK] WRONG SYMLINK",
        Reasons::InvalidGfid => "[NOT OK] INVALID GFID ",
        Reasons::InvalidPGfid => "[NOT OK] INVALID PGFID",
    };
    println!("{} {:36} {:36} {}",
             ok_msg,
             gfid,
             pgfid,
             entry.path().display());
}

// path_is_symbolic_link

fn get_paths(path: &str) {
    println!("STATUS   DESCRIPTION   {:36} {:36} PATH", "GFID", "PGFID");
    println!("-------- ------------- ------------------------------------ \
              ------------------------------------ ---------------------");

    // Recursive crawl
    for entry in WalkDir::new(path).into_iter().filter_entry(|e| !filter_dirs(e)) {
        let entry = entry.unwrap();

        // Ignore Brick Root
        if entry.path().to_str().unwrap() == path {
            continue;
        }

        // If it is a directory
        if entry.file_type().is_dir() {
            // Collect the GFID of dir
            let gfid = match get_gfid(entry.path().to_str().unwrap()) {
                Ok(v) => {
                    if v == "".to_string() {
                        output_display(Reasons::InvalidGfid, &entry, "", "");
                        continue;
                    }
                    v
                }
                Err(_) => {
                    output_display(Reasons::NoGfid, &entry, "", "");
                    continue;
                }
            };

            // Construct the gfid path as
            // $BRICK/.glusterfs/<gfid[0:2]>/<gfid[2:4]>/<gfid>
            let gfid_path = Path::new(path)
                .join(".glusterfs")
                .join(&gfid[0..2])
                .join(&gfid[2..4])
                .join(&gfid);

            // Get Parent Dir GFID
            let pgfid = match get_gfid(entry.path().parent().unwrap().to_str().unwrap()) {
                Ok(v) => {
                    if v == "".to_string() {
                        output_display(Reasons::InvalidPGfid, &entry, &gfid, "");
                        continue;
                    }
                    v
                }
                Err(_) => {
                    output_display(Reasons::NoPgfid, &entry, &gfid, "");
                    continue;
                }
            };

            // Construct the pgfid path to compare with symlink target
            // ../../<pgfid[0:2]>/<pgfid[2:4]>/<pgfid>/<dirname>
            let pgfid_path = Path::new("../../")
                .join(&pgfid[0..2])
                .join(&pgfid[2..4])
                .join(&pgfid)
                .join(entry.file_name().to_str().unwrap());

            // Try reading the symlink, All good if read link matches with expected pgfid_path
            // If target is different then it may have wrongly linked to different directory
            match gfid_path.read_link() {
                Ok(v) => {
                    if v == pgfid_path {
                        output_display(Reasons::Ok, &entry, &gfid, &pgfid);
                    } else {
                        output_display(Reasons::WrongSymlink, &entry, &gfid, &pgfid);
                    }
                }
                Err(_) => {
                    output_display(Reasons::NoSymlink, &entry, &gfid, &pgfid);
                }
            }
        }
    }
}

fn main() {
    let mut args = env::args();
    match args.nth(1) {
        Some(v) => get_paths(&v),
        None => println!("Brick path is required"),
    }
}
