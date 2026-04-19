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

/// Creates a timestamped version snapshot of the current file before updating.
pub fn save_version(wiki_root: &Path, file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        return Ok(()); // Nothing to version if it doesn't exist
    }
    
    let version_dir = get_version_dir(wiki_root, file_path)?;
    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
    let version_path = version_dir.join(format!("{}.md", timestamp));
    
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
