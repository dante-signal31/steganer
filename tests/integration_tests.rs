use std::env::current_dir;
use std::fs::read;
use std::path::Path;

use steganer::run;
use steganer::create_configuration;
use test_common::{copy_files, hash_file, TestEnvironment};

const SOURCE_FOLDER: &str = "tests/resources/";
const HIDDEN_FILE: &str = "loren.txt";
//const HIDDEN_FILE: &str = "character.txt";
// By Original full portrait: "Playmate of the Month". Playboy Magazine. November 1972,
// photographed by Dwight Hooker. This 512x512 electronic/mechanical scan of a section of the
// full portrait: Alexander Sawchuk and two others[1] - The USC-SIPI image database,
// Fair use, https://en.wikipedia.org/w/index.php?curid=20658476
const HOST_FILE: &str = "Lenna_(test_image).png";
const FILE_RECOVERED: &str = "lenna_recovered.txt";

#[test]
fn test_simple_hiding() {
    // Create test temp folder and populate it with test files.
    let test_folder = TestEnvironment::new();
    let test_folder_path = test_folder.path();
    let current_folder = current_dir()
        .expect("Error obtaining current working folder");
    let current_folder_path = Path::new(current_folder.as_path());
    let file_hidden_absolute_path = current_folder_path.join(SOURCE_FOLDER).join(HIDDEN_FILE)
        .into_os_string().into_string()
        .expect("File to hide name has non valid unicode characters.");
    let host_file_absolute_path = current_folder_path.join(SOURCE_FOLDER).join(HOST_FILE)
        .into_os_string().into_string()
        .expect("Host file name has not valid unicode characters.");
    let files_to_copy: Vec<&str> = vec![file_hidden_absolute_path.as_str(), host_file_absolute_path.as_str()];
    copy_files(files_to_copy, test_folder_path.to_str()
        .expect("Test folder path contains non valid unicode characters that made conversion impossible."));
    let test_hidden_file = test_folder_path.join(HIDDEN_FILE).into_os_string().into_string()
        .expect("Hidden file name has no valid unicode characters");
    let test_host_file = test_folder_path.join(HOST_FILE).into_os_string().into_string()
        .expect("Host file name has no valid unicode characters");
    // Start test.
    // Check hiding does not raise any error.
    let hiding_config = create_configuration(test_hidden_file.as_str(),
                                                  test_host_file.as_str(),
                                                  false);
    assert_eq!((), run(&hiding_config).expect("Error happened"));
    // Check extraction does not raise any error.
    let recovered_file_absolute_path = test_folder_path.join(FILE_RECOVERED).into_os_string().into_string()
        .expect("Error generating recovered file absolute path.");
    let host_file_loaded_absolute_path = test_folder_path.join(HOST_FILE).into_os_string().into_string()
        .expect("Error generating host file loaded absolute path");
    let extraction_config = create_configuration(recovered_file_absolute_path.as_str(),
                                                 host_file_loaded_absolute_path.as_str(),
                                                 true);
    assert_eq!((), run(&extraction_config).expect("Error happened"));

    // Test destination file has same content than source file.
    let original_file_hash = hash_file(file_hidden_absolute_path.as_str())
        .expect("Something wrong happened when calculating hash for source file.");
    let recovered_file_hash = hash_file(recovered_file_absolute_path.as_str())
        .expect("Something wrong happened when calculating hash for destination file.");
    assert_eq!(original_file_hash.as_ref(), recovered_file_hash.as_ref(),
               "Recovered file content is not the same as original file content. \
                Original hash is {:X?} and recovered. is {:X?}",
               original_file_hash.as_ref(), recovered_file_hash.as_ref());

}