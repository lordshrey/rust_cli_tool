use clap::{App, Arg};
use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;
use url::Url;

/// A simple wget-like CLI tool for downloading files from URLs.
///
/// This program allows users to download files from specified URLs and optionally
/// save them with custom filenames.
///
/// # Usage
///
/// ```
/// rustwget [OPTIONS] <URL>
/// ```
///
/// # Arguments
///
/// * `<URL>`: The URL of the file to download (required)
///
/// # Options
///
/// * `-O, --output <FILE>`: Specify a custom filename for the downloaded file
///
/// # Examples
///
/// ```
/// rustwget https://example.com/file.txt
/// rustwget -O custom_name.txt https://example.com/file.txt
/// ```

/// The main function that sets up the CLI and initiates the download process.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>`: Ok(()) if successful, or an error if something goes wrong.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("rustwget")
        .version("1.0")
        .author("AskCodi")
        .about("A simple wget-like CLI tool")
        .arg(
            Arg::with_name("URL")
                .help("The URL to download")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .short("O")
                .long("output")
                .value_name("FILE")
                .help("Write documents to FILE")
                .takes_value(true),
        )
        .get_matches();

    let url = matches.value_of("URL").unwrap();
    let output = matches.value_of("output");

    let client = Client::new();

    download_file(&client, url, output)
}

/// Downloads a file from the specified URL and saves it to the local filesystem.
///
/// # Arguments
///
/// * `client`: A reference to the HTTP client used for making requests.
/// * `url`: The URL of the file to download.
/// * `output`: An optional custom filename for the downloaded file.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>`: Ok(()) if the download is successful, or an error if something goes wrong.
///
/// # Errors
///
/// This function can return errors in the following cases:
/// * If the HTTP request fails
/// * If the server returns a non-success status code
/// * If there's an issue creating or writing to the output file
/// * If the URL parsing fails
fn download_file(client: &Client, url: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading: {}", url);

    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to download: HTTP {}", response.status()).into());
    }

    let url = Url::parse(url)?;
    
    let filename = output.unwrap_or_else(|| {
        url.path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("index.html")
    });

    let mut file = File::create(filename)?;
    let content = response.bytes()?;
    file.write_all(&content)?;

    println!("Downloaded: {}", filename);

    Ok(())
}

/// Test module for the download functionality.
///
/// This module contains unit tests to verify the behavior of the `download_file` function
/// under various scenarios, including successful downloads, invalid URLs, HTTP errors,
/// and different output filename options.
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_successful_download() {
        let content = "Hello, World!";
        let mock = mock("GET", "/file.txt")
            .with_status(200)
            .with_body(content)
            .create();

        let url = format!("{}/file.txt", server_url());
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        let client = Client::new();
        let result = download_file(&client, &url, Some(output_path));

        assert!(result.is_ok());

        let mut file_content = String::new();
        let mut file = File::open(output_path).unwrap();
        file.read_to_string(&mut file_content).unwrap();

        assert_eq!(file_content, content);
        mock.assert();
    }

    #[test]
    fn test_invalid_url() {
        let client = Client::new();
        let invalid_url = "not_a_valid_url";

        let result = download_file(&client, invalid_url, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("relative URL without a base"));
    }

    #[test]
    fn test_http_error() {
        let mock = mock("GET", "/not_found")
            .with_status(404)
            .create();

        let url = format!("{}/not_found", server_url());
        let client = Client::new();

        let result = download_file(&client, &url, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to download: HTTP 404"));
        mock.assert();
    }

    #[test]
    fn test_custom_output_filename() {
        let content = "Custom filename content";
        let mock = mock("GET", "/custom_file.txt")
            .with_status(200)
            .with_body(content)
            .create();

        let url = format!("{}/custom_file.txt", server_url());
        let temp_file = NamedTempFile::new().unwrap();
        let custom_filename = temp_file.path().to_str().unwrap();

        let client = Client::new();
        let result = download_file(&client, &url, Some(custom_filename));

        assert!(result.is_ok());

        let mut file_content = String::new();
        let mut file = File::open(custom_filename).unwrap();
        file.read_to_string(&mut file_content).unwrap();

        assert_eq!(file_content, content);
        mock.assert();
    }

    #[test]
    fn test_default_output_filename() {
        let content = "Default filename content";
        let mock = mock("GET", "/default_file.txt")
            .with_status(200)
            .with_body(content)
            .create();

        let url = format!("{}/default_file.txt", server_url());
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let client = Client::new();
        let result = download_file(&client, &url, None);

        assert!(result.is_ok());

        let default_filename = "default_file.txt";
        assert!(temp_dir.path().join(default_filename).exists());

        let mut file_content = String::new();
        let mut file = File::open(default_filename).unwrap();
        file.read_to_string(&mut file_content).unwrap();

        assert_eq!(file_content, content);
        mock.assert();
    }
}
