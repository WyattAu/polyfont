use std::path::{Path, PathBuf};

use crate::{FontError, Result};

#[derive(Debug, Clone)]
/// Source location for downloadable fonts.
pub enum FontSource {
    GitHub { repo: String, tag: Option<String> },
    GoogleFonts,
    Url(String),
}

#[derive(Debug, Clone)]
/// Metadata about a known downloadable font.
pub struct FontSourceInfo {
    pub name: String,
    pub source: FontSource,
    pub license: String,
}

static KNOWN_FONTS: std::sync::LazyLock<Vec<FontSourceInfo>> = std::sync::LazyLock::new(|| {
    vec![
        FontSourceInfo {
            name: "JetBrains Mono".to_string(),
            source: FontSource::GitHub {
                repo: "JetBrains/JetBrainsMono".to_string(),
                tag: None,
            },
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Fira Code".to_string(),
            source: FontSource::GitHub {
                repo: "tonsky/FiraCode".to_string(),
                tag: None,
            },
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Maple Mono".to_string(),
            source: FontSource::GitHub {
                repo: "subframe7536/maple-font".to_string(),
                tag: None,
            },
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "IBM Plex Mono".to_string(),
            source: FontSource::GitHub {
                repo: "IBM/plex".to_string(),
                tag: None,
            },
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Source Code Pro".to_string(),
            source: FontSource::GitHub {
                repo: "adobe-fonts/source-code-pro".to_string(),
                tag: None,
            },
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Monaspace".to_string(),
            source: FontSource::GitHub {
                repo: "githubnext/monaspace".to_string(),
                tag: None,
            },
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Source Serif Pro".to_string(),
            source: FontSource::GoogleFonts,
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Source Serif 4".to_string(),
            source: FontSource::GoogleFonts,
            license: "OFL-1.1".to_string(),
        },
        FontSourceInfo {
            name: "Roboto Mono".to_string(),
            source: FontSource::GoogleFonts,
            license: "Apache-2.0".to_string(),
        },
    ]
});

/// Downloads and caches font files from remote sources.
pub struct FontDownloader {
    cache_dir: PathBuf,
    lock_path: PathBuf,
}

impl FontDownloader {
    pub fn new() -> Self {
        let cache_dir = Self::default_cache_dir();
        let lock_path = cache_dir.join("fonts.lock.toml");
        Self {
            cache_dir,
            lock_path,
        }
    }

    pub fn default_cache_dir() -> PathBuf {
        if cfg!(target_os = "windows") {
            let base = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
            PathBuf::from(base).join("polyfont").join("fonts")
        } else {
            let cache_base = std::env::var("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    std::env::var("HOME")
                        .map(|h| PathBuf::from(h).join(".cache"))
                        .unwrap_or_else(|_| PathBuf::from("/tmp/.cache"))
                });
            cache_base.join("polyfont").join("fonts")
        }
    }

    pub fn download_to_cache(&self, family: &str, source: FontSource) -> Result<PathBuf> {
        #[cfg(feature = "download")]
        {
            self.download_to_cache_impl(family, source)
        }
        #[cfg(not(feature = "download"))]
        {
            let _ = (family, source);
            Err(FontError::DiscoveryFailed(
                "font download requires 'download' feature".to_string(),
            ))
        }
    }

    pub fn list_available_sources() -> Vec<FontSourceInfo> {
        KNOWN_FONTS.clone()
    }

    pub fn verify_checksum(path: &Path, expected: &str) -> Result<bool> {
        #[cfg(feature = "download")]
        {
            Self::verify_checksum_impl(path, expected)
        }
        #[cfg(not(feature = "download"))]
        {
            let _ = (path, expected);
            Err(FontError::DiscoveryFailed(
                "checksum verification requires 'download' feature".to_string(),
            ))
        }
    }
}

impl Default for FontDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "download")]
#[derive(Debug, Clone)]
struct LockEntry {
    family: String,
    version: String,
    sha256: String,
    path: PathBuf,
    downloaded_at: String,
}

#[cfg(feature = "download")]
impl FontDownloader {
    fn ensure_cache_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    fn compute_sha256(path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let data = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn verify_checksum_impl(path: &Path, expected: &str) -> Result<bool> {
        let actual = Self::compute_sha256(path)?;
        Ok(actual == expected)
    }

    fn read_lockfile(&self) -> Result<Vec<LockEntry>> {
        if !self.lock_path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&self.lock_path)?;
        let table: toml::Table =
            toml::from_str(&content).map_err(|e| FontError::Lockfile(e.to_string()))?;
        let entries = table
            .get("entry")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let t = item.as_table()?;
                        Some(LockEntry {
                            family: t.get("family")?.as_str()?.to_string(),
                            version: t.get("version")?.as_str()?.to_string(),
                            sha256: t.get("sha256")?.as_str()?.to_string(),
                            path: PathBuf::from(t.get("path")?.as_str()?),
                            downloaded_at: t.get("downloaded_at")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(entries)
    }

    fn write_lockfile(&self, entries: &[LockEntry]) -> Result<()> {
        let mut table = toml::map::Map::new();
        let arr: Vec<toml::Value> = entries
            .iter()
            .map(|e| {
                let mut entry = toml::map::Map::new();
                entry.insert("family".to_string(), toml::Value::String(e.family.clone()));
                entry.insert(
                    "version".to_string(),
                    toml::Value::String(e.version.clone()),
                );
                entry.insert("sha256".to_string(), toml::Value::String(e.sha256.clone()));
                entry.insert(
                    "path".to_string(),
                    toml::Value::String(e.path.to_string_lossy().to_string()),
                );
                entry.insert(
                    "downloaded_at".to_string(),
                    toml::Value::String(e.downloaded_at.clone()),
                );
                toml::Value::Table(entry)
            })
            .collect();
        table.insert("entry".to_string(), toml::Value::Array(arr));
        let content =
            toml::to_string_pretty(&table).map_err(|e| FontError::Lockfile(e.to_string()))?;
        std::fs::write(&self.lock_path, content)?;
        Ok(())
    }

    fn find_by_checksum(&self, sha256: &str) -> Option<LockEntry> {
        self.read_lockfile()
            .ok()?
            .into_iter()
            .find(|e| e.sha256 == sha256)
    }

    fn download_to_cache_impl(&self, family: &str, source: FontSource) -> Result<PathBuf> {
        self.ensure_cache_dir()?;

        let url = match &source {
            FontSource::Url(u) => u.clone(),
            FontSource::GitHub { repo, tag } => {
                let tag = tag.as_deref().unwrap_or("latest");
                let slug = family.replace(' ', "");
                format!("https://github.com/{repo}/releases/download/{tag}/{slug}.zip")
            }
            FontSource::GoogleFonts => {
                let encoded: String = family.replace(' ', "+");
                format!("https://fonts.google.com/download?family={encoded}")
            }
        };

        let client = reqwest::blocking::Client::builder()
            .user_agent("polyfont")
            .build()
            .map_err(|e| {
                FontError::DiscoveryFailed(format!("failed to create HTTP client: {e}"))
            })?;

        let response = client
            .get(&url)
            .send()
            .map_err(|e| FontError::DiscoveryFailed(format!("download failed: {e}")))?;

        if !response.status().is_success() {
            return Err(FontError::DiscoveryFailed(format!(
                "download failed with status: {}",
                response.status()
            )));
        }

        let is_zip_by_content = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.contains("zip"));
        let is_zip = url.ends_with(".zip") || is_zip_by_content;

        let bytes = response.bytes().map_err(|e| {
            FontError::DiscoveryFailed(format!("failed to read response body: {e}"))
        })?;

        let paths = if is_zip {
            self.extract_fonts(&bytes)?
        } else {
            let ext = if url.ends_with(".otf") { "otf" } else { "ttf" };
            let filename = format!("{}.{}", family.replace(' ', "_"), ext);
            let dest = self.cache_dir.join(&filename);
            std::fs::write(&dest, &bytes)?;
            vec![dest]
        };

        let primary = paths
            .first()
            .ok_or_else(|| {
                FontError::DiscoveryFailed("no font files found in archive".to_string())
            })?
            .clone();

        let sha256 = Self::compute_sha256(&primary)?;

        if let Some(existing) = self.find_by_checksum(&sha256) {
            return Ok(existing.path);
        }

        let version = match &source {
            FontSource::GitHub { tag, .. } => tag.clone().unwrap_or_else(|| "latest".to_string()),
            _ => "unknown".to_string(),
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut entries = self.read_lockfile()?;
        entries.push(LockEntry {
            family: family.to_string(),
            version,
            sha256,
            path: primary.clone(),
            downloaded_at: timestamp.to_string(),
        });
        self.write_lockfile(&entries)?;

        Ok(primary)
    }

    fn extract_fonts(&self, data: &[u8]) -> Result<Vec<PathBuf>> {
        use std::io::Read;

        let reader = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(reader)
            .map_err(|e| FontError::DiscoveryFailed(format!("failed to read zip archive: {e}")))?;

        let mut paths = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                FontError::DiscoveryFailed(format!("failed to read zip entry: {e}"))
            })?;

            let name = file.name().to_string();
            if name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc") {
                let file_name = Path::new(&name)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let dest = self.cache_dir.join(&file_name);
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                std::fs::write(&dest, &buf)?;
                paths.push(dest);
            }
        }

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_fonts_registry_not_empty() {
        let sources = FontDownloader::list_available_sources();
        assert!(!sources.is_empty());
        assert!(sources.iter().any(|s| s.name == "JetBrains Mono"));
        assert!(sources.iter().any(|s| s.name == "Fira Code"));
        assert!(sources.iter().any(|s| s.name == "Monaspace"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_cache_dir_path_linux() {
        let dir = FontDownloader::default_cache_dir();
        let dir_str = dir.to_string_lossy();
        assert!(
            dir_str.contains("polyfont"),
            "cache path should contain 'polyfont': {dir_str}"
        );
        assert!(
            dir_str.contains("fonts"),
            "cache path should contain 'fonts': {dir_str}"
        );
    }

    #[test]
    fn test_fonts_lock_toml_parse() {
        let toml_str = r#"
[[entry]]
family = "JetBrains Mono"
version = "2.304"
sha256 = "abc123def456"
path = "/home/user/.cache/polyfont/fonts/JetBrainsMono-Regular.ttf"
downloaded_at = "1700000000"
"#;
        let table: toml::Table = toml::from_str(toml_str).unwrap();
        let entries = table.get("entry").unwrap().as_array().unwrap();
        assert_eq!(entries.len(), 1);
        let entry = entries[0].as_table().unwrap();
        assert_eq!(entry["family"].as_str(), Some("JetBrains Mono"));
        assert_eq!(entry["sha256"].as_str(), Some("abc123def456"));
        assert_eq!(
            entry["path"].as_str(),
            Some("/home/user/.cache/polyfont/fonts/JetBrainsMono-Regular.ttf")
        );
    }

    #[test]
    fn test_google_fonts_url_construction() {
        let sources = FontDownloader::list_available_sources();
        let google_fonts: Vec<_> = sources
            .iter()
            .filter(|s| matches!(s.source, FontSource::GoogleFonts))
            .collect();
        assert!(
            !google_fonts.is_empty(),
            "should have at least one Google Fonts entry"
        );

        for gf in &google_fonts {
            if let FontSource::GoogleFonts = gf.source {
                let encoded = gf.name.replace(' ', "+");
                let url = format!("https://fonts.google.com/download?family={encoded}");
                assert!(
                    url.contains(&gf.name.replace(' ', "+")),
                    "URL should contain encoded family name"
                );
            }
        }
    }

    #[test]
    #[cfg(feature = "download")]
    fn test_verify_checksum_valid() {
        let dir = std::env::temp_dir().join("polyfont_test_checksum");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.bin");
        std::fs::write(&file_path, b"hello world").unwrap();

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"hello world");
        let expected = format!("{:x}", hasher.finalize());

        assert!(
            FontDownloader::verify_checksum(&file_path, &expected).unwrap(),
            "checksum should match for known content"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
