//! Runtime configuration stub

#[derive(Debug, Clone, Default)]
pub struct RuntimeFeatureConfig {
    pub auto_compaction: bool,
}

impl RuntimeFeatureConfig {
    pub fn default_features() -> Self {
        Self {
            auto_compaction: true,
        }
    }
}
