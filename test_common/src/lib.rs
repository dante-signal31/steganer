use std::fs::{remove_file, copy};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Error};
use std::path::Path;

use ring::digest::{Context, Digest, SHA256};

use tempfile::{tempdir, TempDir};

/// Context manager like struct to create temporal folder to perform tests inside.
///
/// TempDir type is stored in private attribute folder. TempDir removes generated temp folder
/// and its contents when it detects it es falling out of scope, So you do not need to remove
/// manually generated temp folder.
///
/// # Example
/// ```rust
/// use crate::test_common::TestEnvironment;
///
/// {
///     let test_folder = TestEnvironment::new();
///     let test_folder_path = test_folder.path();
///     // Do your operations in test folder.
/// } // Here test folder is automatically removed.
/// ```
pub struct TestEnvironment {
    folder: TempDir,
}

// TempDir automatically removes generated test folder, so implementing Drop trait is not needed.
impl TestEnvironment {
    #[must_use]
    pub fn new()-> Self {
        let temp_folder = tempdir().expect("Could not create a temporal test environment.");
        TestEnvironment{folder: temp_folder}
    }

    /// Return a Path reference to generated test environment.
    pub fn path(&self)-> &Path{
        self.folder.path()
    }
}

/// Delete an specific file.
///
/// Returns Ok(()) if sucessful and std::io::Error if not.
///
/// # Parameters:
/// * file_path: &str with the absolute path to file.
pub fn delete_file(file_path: &str)-> Result<(), io::Error>{
    remove_file(file_path)?;
    Ok(())
}

/// Delete all files set in given list.
///
/// Returns an io::Error if any file does not exists unless ignore_missing was true.
///
/// # Parameters:
/// * files: Vector with filepath list to remove.
/// * ignore_missing: If true does not return an error if any of files actually does not exists.
pub fn delete_files(files: Vec<String>, ignore_missing: bool)-> Result<(), io::Error>{
    for file in files{
        match delete_file(file.as_str()){
            Ok(_)=> { continue; },
            Err(e)=> {
                if ignore_missing { continue; } else { e }
            }
        };
    }
    Ok(())
}

/// Copy an specific file.
///
/// Returns an Ok(u64) with copied file size if operation was successful. Otherwise
/// it returns an io::Error.
///
/// # Parameters:
/// * source_file_path: &str with absolute pathname to original file.
/// * destination_file_path: &str with absolute pathname to copied file.
pub fn copy_file(source_file_path: &str, destination_file_path: &str)-> Result<u64, io::Error>{
    Ok(copy(source_file_path, destination_file_path)?)
}

/// Copy all files in an given list to a given destination folder. Original file names
/// are kept untouched.
pub fn copy_files(files: Vec<String>, destination_folder_path: &str)-> Result<(), io::Error>{
    for file in files{
        let path = Path::new(&file);
        if let Some(filename) = path.file_name() {
            let destination_filename = Path::new(destination_folder_path).join(filename);
            copy_file(file.as_str(), destination_filename.as_path().to_str()
                .expect("Destination filen name for copy has non valid unicode characters."))?;
        }
    }
    Ok(())
}

/// Hash file content with SHA-256.
///
/// This way we can check to files have same content.
///
/// Original code got from [Rust Cookbok](https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html)
pub fn hash_file(file_path: &str) -> Result<Digest, Error> {
    let mut reader = BufReader::new(File::open(file_path)?);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}