mod common;

use steganer::run;
use steganer::create_configuration;
use common::delete_files;

const FILE_HIDDEN: String = "resources/loren.txt".to_owned();
const HOST_FILE: String = "resources/lena.png".to_owned();
const HOST_FILE_LOADED: String = "resources/lena_steg.png".to_owned();
const FILE_RECOVERED: String = "resources/lena_recovered.txt".to_owned();

#[test]
fn test_simple_compression() {
    delete_files(vec!(HOST_FILE_LOADED, FILE_RECOVERED), true);
    let compression_config = create_configuration(FILE_HIDDEN,
                                          HOST_FILE,
                                          false);
    // Check execution does not raise any error.
    assert_eq!(Ok(()),run(compression_config));
    let extraction_config = create_configuration(FILE_RECOVERED,
                                                 HOST_FILE_LOADED,
                                                 true);
    let original_content = // TODO read FILE_HIDDEN.
    let recovered_content = // TODO read FILE_RECOVERED.
    // Check what we recovered is what we hid.
    assert_eq!(original_content, recovered_content);
}