use anyhow::{Context, Result};
use chrono::Utc;
use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub uid: Option<String>,
    pub title: Option<String>,
    pub created_by: Option<String>,
    pub updated_at: Option<String>,
}

pub struct WikiPage {
    pub meta: WikiPageMeta,
    pub content: String,
}

/// Parses a Markdown file with YAML frontmatter.
pub fn parse_page(content: &str) -> Result<WikiPage> {
    let matter = Matter::<YAML>::new();
    let (meta, content) = match matter.parse::<WikiPageMeta>(content) {
        Ok(parsed) => {
            let meta = parsed.data.unwrap_or_else(|| WikiPageMeta {
                uid: None,
                title: None,
                created_by: None,
                updated_at: None,
            });
            (meta, parsed.content)
        },
        Err(_) => {
            let meta = WikiPageMeta {
                uid: None,
                title: None,
                created_by: None,
                updated_at: None,
            };
            (meta, content.to_string())
        }
    };

    Ok(WikiPage {
        meta,
        content,
    })
}

/// Helper to serialize meta and content back to a string
pub fn assemble_page(meta: &WikiPageMeta, content: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(meta)?;
    Ok(format!("---\n{}---\n{}", yaml, content))
}

fn version_dir_key(wiki_root: &Path, file_path: &Path) -> String {
    let relative_path = file_path.strip_prefix(wiki_root).unwrap_or(file_path);
    let normalized = relative_path
        .components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::ParentDir => Some("..".to_string()),
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            Component::RootDir => Some("/".to_string()),
            Component::Prefix(prefix) => Some(prefix.as_os_str().to_string_lossy().into_owned()),
        })
        .collect::<Vec<_>>()
        .join("/");

    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn get_version_dir(wiki_root: &Path, file_path: &Path) -> Result<PathBuf> {
    let version_key = version_dir_key(wiki_root, file_path);

    let wiki_type = wiki_root.file_name().unwrap_or_default().to_string_lossy();
    let dot_dir = format!(".exmind-{}", wiki_type);

    let version_dir = wiki_root.join(&dot_dir).join("versions").join(version_key);
    fs::create_dir_all(&version_dir)?;

    Ok(version_dir)
}

fn get_unique_version_path(version_dir: &Path) -> PathBuf {
    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S%.9fZ").to_string();
    let mut version_path = version_dir.join(format!("{}.md", timestamp));

    if !version_path.exists() {
        return version_path;
    }

    let mut counter = 1;
    loop {
        version_path = version_dir.join(format!("{}-{}.md", timestamp, counter));
        if !version_path.exists() {
            return version_path;
        }
        counter += 1;
    }
}

/// Creates a timestamped version snapshot of the current file before updating.
pub fn save_version(wiki_root: &Path, file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        return Ok(()); // Nothing to version if it doesn't exist
    }
    
    let version_dir = get_version_dir(wiki_root, file_path)?;
    let version_path = get_unique_version_path(&version_dir);
    
    fs::copy(file_path, &version_path)?;
    
    Ok(())
}

/// Updates a page, creating a version snapshot of the old content first.
pub fn update_page(wiki_root: &Path, file_path: &Path, meta: &WikiPageMeta, content: &str) -> Result<()> {
    save_version(wiki_root, file_path)?;
    
    let full_content = assemble_page(meta, content)?;
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, full_content)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}", prefix, nanos))
    }

    fn cleanup_dir(path: &Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    fn parse_and_assemble_round_trip_with_frontmatter() {
        let meta = WikiPageMeta {
            uid: Some("page-123".to_string()),
            title: Some("Hello".to_string()),
            created_by: Some("tester".to_string()),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let body = "# Heading\n\nSome content.";

        let assembled = assemble_page(&meta, body).expect("assemble_page should succeed");
        let parsed = parse_page(&assembled).expect("parse_page should succeed");

        assert_eq!(parsed.meta.uid, meta.uid);
        assert_eq!(parsed.meta.title, meta.title);
        assert_eq!(parsed.meta.created_by, meta.created_by);
        assert_eq!(parsed.meta.updated_at, meta.updated_at);
        assert_eq!(parsed.content, body);
    }

    #[test]
    fn parse_page_falls_back_to_plain_content_on_invalid_yaml() {
        let invalid = "---\n: invalid yaml\n---\nBody stays intact";

        let parsed = parse_page(invalid).expect("parse_page should fall back on invalid YAML");

        assert!(parsed.meta.uid.is_none());
        assert!(parsed.meta.title.is_none());
        assert!(parsed.meta.created_by.is_none());
        assert!(parsed.meta.updated_at.is_none());
        assert_eq!(parsed.content, invalid);
    }

    #[test]
    fn save_version_uses_sanitized_version_directory_layout() {
        let wiki_root = unique_test_dir("wiki_root_layout");
        let file_path = wiki_root.join("nested").join("dir").join("page.md");

        fs::create_dir_all(file_path.parent().expect("file path should have a parent"))
            .expect("failed to create page parent directory");
        fs::write(&file_path, "original page contents").expect("failed to write source page");

        save_version(&wiki_root, &file_path).expect("save_version should succeed");

        let expected_version_dir = wiki_root
            .join(".exmind-wiki_root_layout")
            .join("versions")
            .join("nested_dir_page.md");

        assert!(expected_version_dir.is_dir(), "version directory should be created");

        let entries: Vec<PathBuf> = fs::read_dir(&expected_version_dir)
            .expect("failed to read version directory")
            .map(|entry| entry.expect("failed to read version entry").path())
            .collect();

        assert_eq!(entries.len(), 1, "expected a single saved snapshot");
        assert_eq!(
            fs::read_to_string(&entries[0]).expect("failed to read saved snapshot"),
            "original page contents"
        );
        assert_eq!(entries[0].extension().and_then(|ext| ext.to_str()), Some("md"));

        cleanup_dir(&wiki_root);
    }

    #[test]
    fn save_version_multiple_times_keeps_snapshots_in_expected_directory() {
        let wiki_root = unique_test_dir("wiki_root_multi");
        let file_path = wiki_root.join("windows\\style\\name.md");

        fs::create_dir_all(&wiki_root).expect("failed to create wiki root");
        fs::write(&file_path, "v1").expect("failed to write initial file");

        save_version(&wiki_root, &file_path).expect("first save_version should succeed");
        fs::write(&file_path, "v2").expect("failed to update source file");
        save_version(&wiki_root, &file_path).expect("second save_version should succeed");

        let expected_version_dir = wiki_root
            .join(".exmind-wiki_root_multi")
            .join("versions")
            .join("windows_style_name.md");

        assert!(expected_version_dir.is_dir(), "sanitized version directory should exist");

        let entries: Vec<PathBuf> = fs::read_dir(&expected_version_dir)
            .expect("failed to read version directory")
            .map(|entry| entry.expect("failed to read version entry").path())
            .collect();

        assert!(
            !entries.is_empty(),
            "at least one snapshot should exist after repeated saves"
        );
        assert!(
            entries.iter().all(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md")),
            "all snapshots should use the .md extension"
        );

        cleanup_dir(&wiki_root);
    }
}
