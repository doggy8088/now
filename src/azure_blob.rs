use crate::config::NowConfig;
use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use std::fs;
use std::path::{Component, Path};
use url::Url;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AzureBlobUploadSummary {
    pub files: usize,
    pub bytes: u64,
}

pub fn display_upload_command(config: &NowConfig, source: &Path) -> Result<String> {
    let sas_url = sas_url(config)?;
    Ok(format!(
        "Azure Storage Blob SAS upload {} -> {}",
        source.display(),
        mask_sas_url(sas_url)?
    ))
}

pub fn upload_directory(config: &NowConfig, source: &Path) -> Result<AzureBlobUploadSummary> {
    upload_directory_to_sas_url(sas_url(config)?, source)
}

pub fn public_blob_url_for_relative_path(
    config: &NowConfig,
    relative_path: &Path,
) -> Option<String> {
    let mut url = blob_url_for_relative_path(optional_sas_url(config)?, relative_path).ok()?;
    url.set_query(None);
    Some(url.to_string())
}

pub fn public_base_url(config: &NowConfig) -> Option<String> {
    let mut url = Url::parse(optional_sas_url(config)?).ok()?;
    url.set_query(None);
    Some(url.to_string())
}

pub fn upload_directory_to_sas_url(sas_url: &str, source: &Path) -> Result<AzureBlobUploadSummary> {
    if !source.is_dir() {
        bail!(
            "Azure Storage Blob source path is not a directory: {}",
            source.display()
        );
    }

    let client = Client::new();
    let mut summary = AzureBlobUploadSummary { files: 0, bytes: 0 };

    for entry in WalkDir::new(source).sort_by_file_name() {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let relative = entry.path().strip_prefix(source)?;
        let url = blob_url_for_relative_path(sas_url, relative)?;
        let body = fs::read(entry.path())
            .with_context(|| format!("failed to read {}", entry.path().display()))?;
        let content_type = content_type_for_path(entry.path());

        put_blob(&client, url, &body, content_type)
            .with_context(|| format!("failed to upload {}", relative.display()))?;

        summary.files += 1;
        summary.bytes += body.len() as u64;
    }

    Ok(summary)
}

pub fn blob_url_for_relative_path(sas_url: &str, relative_path: &Path) -> Result<Url> {
    let mut url = Url::parse(sas_url).context("azure_blob.sas_url must be a valid URL")?;
    let has_container_path = url
        .path_segments()
        .map(|mut segments| segments.any(|segment| !segment.is_empty()))
        .unwrap_or(false);
    if !has_container_path {
        bail!("azure_blob.sas_url must include a container path");
    }

    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| anyhow!("azure_blob.sas_url cannot be used as a blob base URL"))?;
        segments.pop_if_empty();
        for component in relative_path.components() {
            match component {
                Component::Normal(value) => {
                    let value = value
                        .to_str()
                        .with_context(|| format!("non-UTF-8 path: {}", relative_path.display()))?;
                    segments.push(value);
                }
                Component::CurDir => {}
                _ => bail!("unsupported relative path: {}", relative_path.display()),
            }
        }
    }

    Ok(url)
}

pub fn mask_sas_url(sas_url: &str) -> Result<String> {
    let mut url = Url::parse(sas_url).context("azure_blob.sas_url must be a valid URL")?;
    if url.query().is_some() {
        url.set_query(None);
        Ok(format!("{}?<redacted>", url.as_str()))
    } else {
        Ok(url.to_string())
    }
}

fn put_blob(client: &Client, url: Url, body: &[u8], content_type: &'static str) -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert("x-ms-blob-type", HeaderValue::from_static("BlockBlob"));
    headers.insert("x-ms-version", HeaderValue::from_static("2023-11-03"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

    let response = client
        .put(url)
        .headers(headers)
        .body(body.to_vec())
        .send()?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().unwrap_or_default();
        bail!(
            "Azure Storage Blob upload failed with HTTP {}: {}",
            status.as_u16(),
            text
        );
    }

    Ok(())
}

fn sas_url(config: &NowConfig) -> Result<&str> {
    optional_sas_url(config)
        .context("azure_blob.sas_url is required for provider Azure Storage Blob")
}

fn optional_sas_url(config: &NowConfig) -> Option<&str> {
    config
        .azure_blob
        .sas_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn builds_blob_urls_from_container_sas_url() {
        let url = blob_url_for_relative_path(
            "https://acct.blob.core.windows.net/$web?sv=1&sig=secret",
            Path::new("assets/app.js"),
        )
        .unwrap();

        assert_eq!(
            url.as_str(),
            "https://acct.blob.core.windows.net/$web/assets/app.js?sv=1&sig=secret"
        );
    }

    #[test]
    fn masks_sas_query_in_display_output() {
        let masked =
            mask_sas_url("https://acct.blob.core.windows.net/$web?sv=1&sig=secret").unwrap();

        assert_eq!(masked, "https://acct.blob.core.windows.net/$web?<redacted>");
        assert!(!masked.contains("secret"));
    }

    #[test]
    fn builds_public_blob_urls_without_sas_query() {
        let config = NowConfig {
            azure_blob: crate::config::AzureBlobConfig {
                sas_url: Some(
                    "https://infinitybin.blob.core.windows.net/now/now?sv=1&sig=secret".to_owned(),
                ),
                ..Default::default()
            },
            ..NowConfig::default()
        };

        assert_eq!(
            public_blob_url_for_relative_path(&config, Path::new("index.html")).as_deref(),
            Some("https://infinitybin.blob.core.windows.net/now/now/index.html")
        );
        assert_eq!(
            public_base_url(&config).as_deref(),
            Some("https://infinitybin.blob.core.windows.net/now/now")
        );
    }

    #[test]
    fn uploads_files_with_put_blob_headers() {
        let site = TempDir::new().unwrap();
        site.child("index.html").write_str("<h1>ok</h1>").unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 4096];
            let bytes = stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..bytes]);

            assert!(request.starts_with("PUT /container/index.html?sv=1&sig=secret HTTP/1.1"));
            assert!(request.contains("x-ms-blob-type: BlockBlob"));
            assert!(request.contains("content-type: text/html; charset=utf-8"));

            stream
                .write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
        });

        let sas_url = format!("http://{address}/container?sv=1&sig=secret");
        let summary = upload_directory_to_sas_url(&sas_url, site.path()).unwrap();
        handle.join().unwrap();

        assert_eq!(summary.files, 1);
        assert_eq!(summary.bytes, 11);
    }
}
