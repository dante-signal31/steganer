mod common;

use std::env::current_dir;
use std::fs::read;
use std::path::Path;

use steganer::run;
use steganer::create_configuration;
use common::{copy_files, TestEnvironment};

const FILE_HIDDEN: String = "resources/loren.txt".to_owned();
const HOST_FILE: String = "resources/lena.png".to_owned();
const HOST_FILE_LOADED: String = "lena_steg.png".to_owned();
const FILE_RECOVERED: String = "lena_recovered.txt".to_owned();

#[test]
fn test_simple_compression() {
    // Create test temp folder and populate it with test files.
    let test_folder = TestEnvironment::new();
    let test_folder_path = test_folder.path();
    let current_folder = Path::new(current_dir());
    let file_hidden_absolute_path = current_folder.join(FILE_HIDDEN).into_os_string().into_string()
        .expect("File to hide name has non valid unicode characters.");
    let host_file_absolute_path = current_folder.join(HOST_FILE).into_os_string().into_string()
        .expect("Host filen name has not valid unicode characters.");
    let files_to_copy = vec![file_hidden_absolute_path, host_file_absolute_path];
    copy_files(files_to_copy, test_folder_path.to_str()
        .expect("Test folder path contains non valid unicode characters that made conversion impossible."));

    // Start test.
    // Check compression does not raise any error.
    let compression_config = create_configuration(FILE_HIDDEN,
                                          HOST_FILE,
                                          false);
    assert_eq!(Ok(()),run(compression_config));

    // Check decompression does not raise any error.
    let recovered_file_absolute_path = test_folder_path.join(FILE_RECOVERED).into_os_string().into_string()
        .expect("Error generating recovered file absolute path.");
    let host_file_loaded_absolute_path = test_folder_path.join(HOST_FILE_LOADED).into_os_string().into_string()
        .expect("Error generating host file loaded absolute path");
    let extraction_config = create_configuration(recovered_file_absolute_path,
                                                 host_file_loaded_absolute_path,
                                                 true);
    assert_eq!(Ok(()), run(extraction_config));

    // Check what we recovered is what we hid.
    let original_content = read(file_hidden_absolute_path)
        .expect("Error reading file to hide contents.");
    let recovered_content = read(recovered_file_absolute_path)
        .expect("Error reading recovered file contents.");
    assert_eq!(original_content, recovered_content);
}