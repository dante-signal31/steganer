use std::io::Error;

pub fn delete_file(file_path: String)-> Result<(), Error>{
// TODO: Implement.
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