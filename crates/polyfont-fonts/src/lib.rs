use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;
use tracing::{debug, warn};

pub mod download;

#[derive(Debug, Error)]
pub enum FontError {
    #[error("font command failed: {0}")]
    CommandFailed(String),
    #[error("failed to parse font output")]
    ParseError,
    #[error("font discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("lockfile error: {0}")]
    Lockfile(String),
}

pub type Result<T> = std::result::Result<T, FontError>;

pub trait FontDiscovery: Send + Sync {
    fn list_families(&self) -> Result<Vec<String>>;
    fn find_family(&self, name: &str) -> Result<bool>;
    fn font_path(&self, name: &str) -> Option<PathBuf>;
}

pub struct FcListDiscovery;

impl FcListDiscovery {
    pub fn new() -> Self {
        Self
    }

    fn run_fc_list(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("fc-list")
            .args(args)
            .output()
            .map_err(|e| FontError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(FontError::CommandFailed(format!(
                "fc-list exited with {}",
                output.status
            )));
        }

        String::from_utf8(output.stdout).map_err(|_| FontError::ParseError)
    }

    fn parse_fc_list_families(output: &str) -> Vec<String> {
        output
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                Some(line.split(',').map(|s| s.trim().to_string()))
            })
            .flatten()
            .map(|s| {
                let val = s.split_once(':').map_or(s.as_str(), |(_, v)| v.trim());
                val.trim().to_string()
            })
            .filter(|s| !s.is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

impl FontDiscovery for FcListDiscovery {
    fn list_families(&self) -> Result<Vec<String>> {
        let output = self.run_fc_list(&[":", "family"])?;
        let mut families = Self::parse_fc_list_families(&output);
        families.sort();
        Ok(families)
    }

    fn find_family(&self, name: &str) -> Result<bool> {
        let output = Command::new("fc-match")
            .arg(name)
            .output()
            .map_err(|e| FontError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().starts_with(name) || stdout.contains(name))
    }

    fn font_path(&self, name: &str) -> Option<PathBuf> {
        let output = Command::new("fc-list")
            .args(["family", &format!("={name}"), "file"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().next().and_then(|line| {
            line.split(':').nth(0).and_then(|p| {
                let p = p.trim();
                if p.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(p))
                }
            })
        })
    }
}

impl Default for FcListDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MacOsDiscovery;

impl MacOsDiscovery {
    pub fn new() -> Self {
        Self
    }
}

impl FontDiscovery for MacOsDiscovery {
    fn list_families(&self) -> Result<Vec<String>> {
        let output = Command::new("system_profiler")
            .args(["SPFontsDataType"])
            .output()
            .map_err(|e| FontError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let families = parse_system_profiler_fonts(&stdout);
            if !families.is_empty() {
                let mut sorted = families;
                sorted.sort();
                return Ok(sorted);
            }
            debug!("system_profiler returned no fonts, trying fc-list fallback");
        } else {
            debug!(
                "system_profiler failed, falling back to fc-list: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        FcListDiscovery::new().list_families()
    }

    fn find_family(&self, name: &str) -> Result<bool> {
        let families = self.list_families()?;
        Ok(families.iter().any(|f| {
            f.eq_ignore_ascii_case(name) || f.to_lowercase().contains(&name.to_lowercase())
        }))
    }

    fn font_path(&self, name: &str) -> Option<PathBuf> {
        FcListDiscovery::new().font_path(name)
    }
}

impl Default for MacOsDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_system_profiler_fonts(output: &str) -> Vec<String> {
    let mut families = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if (line.starts_with("Font:") || line.starts_with("Family:"))
            && let Some(name) = line.split_once(':').map(|(_, v)| v.trim())
            && !name.is_empty()
        {
            families.push(name.to_string());
        }
    }

    families
}

pub struct WindowsDiscovery;

impl WindowsDiscovery {
    pub fn new() -> Self {
        Self
    }

    fn query_registry_fonts() -> Result<Vec<String>> {
        let script = r#"
$fonts = Get-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts' -ErrorAction SilentlyContinue
if ($fonts) {
    $fonts.PSObject.Properties |
        Where-Object { $_.Name -notmatch '^PS' } |
        ForEach-Object { $_.Value }
}
"#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|e| FontError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(FontError::CommandFailed(format!(
                "powershell exited with {}",
                output.status
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_powershell_fonts(&stdout))
    }

    fn scan_font_dirs() -> Vec<String> {
        let dirs = [r"C:\Windows\Fonts", r"C:\Users\Public\Fonts"];

        let mut families = Vec::new();

        for dir in &dirs {
            let path = std::path::Path::new(dir);
            if path.is_dir()
                && let Ok(entries) = std::fs::read_dir(path)
            {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc") {
                        let family = name
                            .strip_suffix(".ttf")
                            .or_else(|| name.strip_suffix(".otf"))
                            .or_else(|| name.strip_suffix(".ttc"))
                            .unwrap_or(&name)
                            .to_string();
                        families.push(family);
                    }
                }
            }
        }

        families
    }
}

fn parse_powershell_fonts(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().unwrap_or(line).to_string())
        .collect()
}

impl FontDiscovery for WindowsDiscovery {
    fn list_families(&self) -> Result<Vec<String>> {
        match Self::query_registry_fonts() {
            Ok(families) if !families.is_empty() => {
                let mut sorted = families;
                sorted.sort();
                sorted.dedup();
                Ok(sorted)
            }
            Ok(_) | Err(_) => {
                warn!("Windows registry query failed or empty, scanning font directories");
                let mut families = Self::scan_font_dirs();
                families.sort();
                families.dedup();
                Ok(families)
            }
        }
    }

    fn find_family(&self, name: &str) -> Result<bool> {
        let families = self.list_families()?;
        Ok(families.iter().any(|f| {
            f.eq_ignore_ascii_case(name) || f.to_lowercase().contains(&name.to_lowercase())
        }))
    }

    fn font_path(&self, name: &str) -> Option<PathBuf> {
        let candidates = [r"C:\Windows\Fonts"];

        for dir in &candidates {
            let path = std::path::Path::new(dir);
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let stem = file_name
                        .strip_suffix(".ttf")
                        .or_else(|| file_name.strip_suffix(".otf"))
                        .or_else(|| file_name.strip_suffix(".ttc"))
                        .unwrap_or(&file_name);

                    if stem.eq_ignore_ascii_case(name)
                        || stem.to_lowercase().contains(&name.to_lowercase())
                    {
                        return Some(entry.path());
                    }
                }
            }
        }

        None
    }
}

impl Default for WindowsDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FallbackDiscovery;

impl FallbackDiscovery {
    pub fn new() -> Self {
        Self
    }
}

impl FontDiscovery for FallbackDiscovery {
    fn list_families(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn find_family(&self, _name: &str) -> Result<bool> {
        Ok(false)
    }

    fn font_path(&self, _name: &str) -> Option<PathBuf> {
        None
    }
}

impl Default for FallbackDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_discovery() -> Box<dyn FontDiscovery> {
    if cfg!(target_os = "linux") {
        Box::new(FcListDiscovery)
    } else if cfg!(target_os = "macos") {
        Box::new(MacOsDiscovery)
    } else if cfg!(target_os = "windows") {
        Box::new(WindowsDiscovery)
    } else {
        Box::new(FallbackDiscovery)
    }
}

#[derive(Debug, Clone)]
pub struct FontCheckResult {
    pub available: Vec<String>,
    pub missing: Vec<String>,
}

impl FontCheckResult {
    pub fn all_available(&self) -> bool {
        self.missing.is_empty()
    }
}

pub struct FontScanner {
    discovery: Box<dyn FontDiscovery>,
}

impl FontScanner {
    pub fn new() -> Self {
        Self {
            discovery: create_discovery(),
        }
    }

    pub fn with_discovery(discovery: Box<dyn FontDiscovery>) -> Self {
        Self { discovery }
    }

    pub fn is_available(&self, family: &str) -> bool {
        self.discovery.find_family(family).unwrap_or(false)
    }

    pub fn available_families(&self) -> Vec<String> {
        self.discovery.list_families().unwrap_or_default()
    }

    pub fn check_config(&self, families: &[&str]) -> FontCheckResult {
        let mut available = Vec::new();
        let mut missing = Vec::new();

        for family in families {
            if self.is_available(family) {
                available.push(family.to_string());
            } else {
                missing.push(family.to_string());
            }
        }

        FontCheckResult { available, missing }
    }
}

impl Default for FontScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fc_list_parse_families() {
        let output = "\
DejaVu Sans Mono: DejaVu Sans Mono
Noto Sans Mono CJK SC: Noto Sans Mono CJK SC
Fira Code: Fira Code
Fira Code: Fira Code
Monospace: Monospace
DejaVu Sans Mono: DejaVu Sans Mono
";

        let families = FcListDiscovery::parse_fc_list_families(output);
        assert!(families.contains(&"DejaVu Sans Mono".to_string()));
        assert!(families.contains(&"Noto Sans Mono CJK SC".to_string()));
        assert!(families.contains(&"Fira Code".to_string()));
        assert!(families.contains(&"Monospace".to_string()));
        assert_eq!(families.len(), 4);
    }

    #[test]
    fn test_fc_list_parse_comma_separated() {
        let output = "\
Arial,Helvetica,sans-serif
Times New Roman,Times,serif
";

        let families = FcListDiscovery::parse_fc_list_families(output);
        assert!(families.contains(&"Arial".to_string()));
        assert!(families.contains(&"Helvetica".to_string()));
        assert!(families.contains(&"sans-serif".to_string()));
        assert!(families.contains(&"Times New Roman".to_string()));
        assert!(families.contains(&"Times".to_string()));
        assert!(families.contains(&"serif".to_string()));
    }

    #[test]
    fn test_fc_list_parse_empty() {
        let output = "";
        let families = FcListDiscovery::parse_fc_list_families(output);
        assert!(families.is_empty());
    }

    #[test]
    fn test_fc_list_parse_whitespace_only() {
        let output = "   \n  \n  ";
        let families = FcListDiscovery::parse_fc_list_families(output);
        assert!(families.is_empty());
    }

    #[test]
    fn test_windows_parse_powershell_fonts() {
        let output = "\
Arial (TrueType)
Arial Bold (TrueType)
Consolas (TrueType)
Courier New (TrueType)
";

        let fonts = parse_powershell_fonts(output);
        assert_eq!(fonts.len(), 4);
        assert_eq!(fonts[0], "Arial");
        assert_eq!(fonts[1], "Arial");
        assert_eq!(fonts[2], "Consolas");
        assert_eq!(fonts[3], "Courier");
    }

    #[test]
    fn test_windows_parse_powershell_empty() {
        let fonts = parse_powershell_fonts("");
        assert!(fonts.is_empty());
    }

    #[test]
    fn test_macos_parse_system_profiler() {
        let output = "\
Fonts:

    Font: Arial
      Family: Arial
      Style: Regular

    Font: Courier New
      Family: Courier New
      Style: Bold
";

        let families = parse_system_profiler_fonts(output);
        assert!(families.contains(&"Arial".to_string()));
        assert!(families.contains(&"Courier New".to_string()));
    }

    #[test]
    fn test_macos_parse_empty() {
        let families = parse_system_profiler_fonts("No fonts found");
        assert!(families.is_empty());
    }

    #[test]
    fn test_fallback_returns_empty() {
        let fb = FallbackDiscovery::new();
        assert!(fb.list_families().unwrap().is_empty());
        assert!(!fb.find_family("anything").unwrap());
        assert!(fb.font_path("anything").is_none());
    }

    #[test]
    fn test_auto_detection_selects_platform() {
        let discovery = create_discovery();
        if cfg!(target_os = "linux") {
            let _ = discovery
                .list_families()
                .expect("FcListDiscovery should work on linux");
        } else if cfg!(target_os = "macos") {
            let _ = discovery
                .list_families()
                .expect("MacOsDiscovery should work on macos");
        } else if cfg!(target_os = "windows") {
            let _ = discovery
                .list_families()
                .expect("WindowsDiscovery should work on windows");
        } else {
            assert!(discovery.list_families().unwrap().is_empty());
        }
    }

    struct MockDiscovery {
        families: Vec<String>,
    }

    impl FontDiscovery for MockDiscovery {
        fn list_families(&self) -> Result<Vec<String>> {
            Ok(self.families.clone())
        }

        fn find_family(&self, name: &str) -> Result<bool> {
            Ok(self.families.iter().any(|f| f == name))
        }

        fn font_path(&self, _name: &str) -> Option<PathBuf> {
            None
        }
    }

    #[test]
    fn test_scanner_availability() {
        let mock = MockDiscovery {
            families: vec!["Fira Code".to_string(), "JetBrains Mono".to_string()],
        };
        let scanner = FontScanner::with_discovery(Box::new(mock));

        assert!(scanner.is_available("Fira Code"));
        assert!(scanner.is_available("JetBrains Mono"));
        assert!(!scanner.is_available("Missing Font"));
    }

    #[test]
    fn test_scanner_available_families() {
        let mock = MockDiscovery {
            families: vec!["Arial".to_string(), "Helvetica".to_string()],
        };
        let scanner = FontScanner::with_discovery(Box::new(mock));

        let families = scanner.available_families();
        assert_eq!(families.len(), 2);
    }

    #[test]
    fn test_check_config_result() {
        let mock = MockDiscovery {
            families: vec!["Fira Code".to_string(), "Arial".to_string()],
        };
        let scanner = FontScanner::with_discovery(Box::new(mock));

        let result = scanner.check_config(&["Fira Code", "Arial", "Missing"]);
        assert_eq!(result.available.len(), 2);
        assert_eq!(result.missing.len(), 1);
        assert!(result.available.contains(&"Fira Code".to_string()));
        assert!(result.available.contains(&"Arial".to_string()));
        assert!(result.missing.contains(&"Missing".to_string()));
        assert!(!result.all_available());
    }

    #[test]
    fn test_check_config_all_available() {
        let mock = MockDiscovery {
            families: vec!["Fira Code".to_string()],
        };
        let scanner = FontScanner::with_discovery(Box::new(mock));

        let result = scanner.check_config(&["Fira Code"]);
        assert!(result.all_available());
        assert!(result.missing.is_empty());
    }

    #[test]
    fn test_check_config_empty() {
        let mock = MockDiscovery { families: vec![] };
        let scanner = FontScanner::with_discovery(Box::new(mock));

        let result = scanner.check_config(&[]);
        assert!(result.all_available());
        assert!(result.available.is_empty());
        assert!(result.missing.is_empty());
    }

    #[test]
    fn test_scanner_error_handling() {
        struct ErrorDiscovery;
        impl FontDiscovery for ErrorDiscovery {
            fn list_families(&self) -> Result<Vec<String>> {
                Err(FontError::CommandFailed("test error".to_string()))
            }
            fn find_family(&self, _name: &str) -> Result<bool> {
                Err(FontError::CommandFailed("test error".to_string()))
            }
            fn font_path(&self, _name: &str) -> Option<PathBuf> {
                None
            }
        }

        let scanner = FontScanner::with_discovery(Box::new(ErrorDiscovery));
        assert!(!scanner.is_available("anything"));
        assert!(scanner.available_families().is_empty());
    }
}
