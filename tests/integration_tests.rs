use std::env::current_dir;
//use std::fs::read;
use std::path::Path;

use steganer::_run;
use steganer::_create_configuration;
use test_common::{copy_files, hash_file, TestEnvironment};

const SOURCE_FOLDER: &str = "tests/resources/";
const HIDDEN_FILE: &str = "loren.txt";
// By Original full portrait: "Playmate of the Month". Playboy Magazine. November 1972,
// photographed by Dwight Hooker. This 512x512 electronic/mechanical scan of a section of the
// full portrait: Alexander Sawchuk and two others[1] - The USC-SIPI image database,
// Fair use, https://en.wikipedia.org/w/index.php?curid=20658476
const HOST_FILE_NAME_SUFFIX: &str = "Lenna_(test_image)";
const CORRECT_TESTED_EXTENSIONS: [&str; 3] = ["png", "bmp", "ppm"];
const INCORRECT_TESTED_EXTENSIONS: [&str; 2] = ["jpg", "tga"];
const HOST_FILE: &str = "Lenna_(test_image).png";
const FILE_RECOVERED: &str = "lenna_recovered.txt";

struct TestImages<'a>  {
    extensions: Vec<&'a str>,
    image_name_suffix: &'a str,
}

impl <'a> TestImages<'a> {
    #[must_use]
    fn new(extensions: Vec<&'a str>, image_name_suffix: &'a str)-> Self{
        TestImages {extensions, image_name_suffix}
    }
}

impl <'a> Iterator for TestImages<'a>{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.extensions.pop() {
            Some(current_extension)=> Some(format!("{}.{}",self.image_name_suffix,
                                                        current_extension)),
            None=> None
        }
    }
}

fn hide_test(host_file: &str) {
    // Create test temp folder and populate it with test files.
    let test_folder = TestEnvironment::new();
    let test_folder_path = test_folder.path();
    let current_folder = current_dir()
        .expect("Error obtaining current working folder");
    let current_folder_path = Path::new(current_folder.as_path());
    let file_hidden_absolute_path = current_folder_path.join(SOURCE_FOLDER).join(HIDDEN_FILE)
        .into_os_string().into_string()
        .expect("File to hide name has non valid unicode characters.");
    let host_file_absolute_path = current_folder_path.join(SOURCE_FOLDER).join(host_file)
        .into_os_string().into_string()
        .expect("Host file name has not valid unicode characters.");
    let files_to_copy: Vec<&str> = vec![file_hidden_absolute_path.as_str(), host_file_absolute_path.as_str()];
    copy_files(files_to_copy, test_folder_path.to_str()
        .expect("Test folder path contains non valid unicode characters that made conversion impossible."));
    let test_hidden_file = test_folder_path.join(HIDDEN_FILE).into_os_string().into_string()
        .expect("Hidden file name has no valid unicode characters");
    let test_host_file = test_folder_path.join(host_file).into_os_string().into_string()
        .expect("Host file name has no valid unicode characters");
    // Start test.
    // Check hiding does not raise any error.
    let hiding_config = _create_configuration(test_hidden_file.as_str(),
                                              test_host_file.as_str(),
                                              false);
    assert_eq!((), _run(&hiding_config).expect(format!("Error happened with {}", host_file).as_str()));
    // Check extraction does not raise any error.
    let recovered_file_absolute_path = test_folder_path.join(FILE_RECOVERED).into_os_string().into_string()
        .expect("Error generating recovered file absolute path.");
    let host_file_loaded_absolute_path = test_folder_path.join(host_file).into_os_string().into_string()
        .expect("Error generating host file loaded absolute path");
    let extraction_config = _create_configuration(recovered_file_absolute_path.as_str(),
                                                  host_file_loaded_absolute_path.as_str(),
                                                  true);
    assert_eq!((), _run(&extraction_config).expect(format!("Error happened with {}", host_file).as_str()));

    // Test destination file has same content than source file.
    let original_file_hash = hash_file(file_hidden_absolute_path.as_str())
        .expect("Something wrong happened when calculating hash for source file.");
    let recovered_file_hash = hash_file(recovered_file_absolute_path.as_str())
        .expect("Something wrong happened when calculating hash for destination file.");
    assert_eq!(original_file_hash.as_ref(), recovered_file_hash.as_ref(),
               "Recovered file content is not the same as original file content. \
                Original hash is {:X?} and recovered. is {:X?} \
                after using {} as host image.",
               original_file_hash.as_ref(), recovered_file_hash.as_ref(), host_file);
}

#[test]
fn test_simple_hiding() {
    let correct_images = TestImages::new(CORRECT_TESTED_EXTENSIONS.to_vec(),
                                         HOST_FILE_NAME_SUFFIX);
    for image in correct_images{
        hide_test(image.as_str());
    }
}

#[test]
#[should_panic]
fn test_incorrect_hiding() {
    let incorrect_images = TestImages::new(INCORRECT_TESTED_EXTENSIONS.to_vec(),
                                         HOST_FILE_NAME_SUFFIX);
    for image in incorrect_images {
        hide_test(image.as_str());
    }
}