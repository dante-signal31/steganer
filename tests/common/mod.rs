use std::fs::remove_file;
use std::io::Error;
use std::path::Path;
use tempfile::{tempdir, TempDir};

/// Context manager like struct to create temporal folder to perform tests inside.
///
/// TempDir type is stored in private attribute folder. TempDir removes generated temp folder
/// and its contents when it detects it es falling out of scope, So you do not need to remove
/// manually generated temp folder.
///
/// # Example
/// [...]
/// {
///     let test_folder = TestEnvironment::new();
///     let test_folder_path = test_folder.path();
///     // Do your operations in test folder.
/// } // Here test folder is automatically removed.
struct TestEnvironment {
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
/// * file_path: String with the absolute path to file.
pub fn delete_file(file_path: String)-> Result<(), Error>{
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
pub fn delete_files(files: Vec<String>, ignore_missing: bool)-> Result<(), Error>{
    for file in files{
        match delete_file(file){
            Ok(_)=> { continue; },
            Err(e)=> {
                if ignore_missing { continue; } else { Err(e) }
            }
        };
    }
    Ok(())
}