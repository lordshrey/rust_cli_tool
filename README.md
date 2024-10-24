A simple wget-like CLI tool for downloading files from URLs.

This program allows users to download files from specified URLs and optionally
save them with custom filenames.

# Usage

```
rustwget [OPTIONS] <URL>
```

# Arguments

- `<URL>`: The URL of the file to download (required)

# Options

- `-O, --output <FILE>`: Specify a custom filename for the downloaded file

# Examples

```
rustwget https://example.com/file.txt
rustwget -O custom_name.txt https://example.com/file.txt
```
