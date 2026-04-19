pub mod builtin;
pub mod registry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 技能执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    pub instruction: Option<String>,
    pub original_content: String,
    pub source_path: Option<String>,
    pub target_wiki: Option<String>, // "my-wiki" | "agent-wiki"
}

/// 标准化技能输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    pub proposed_content: String,
    pub diff: Option<String>,
    pub title_suggestion: Option<String>,
    pub meta: Option<Value>,
}

#[async_trait]
pub trait Skill: Send + Sync {
    /// 技能唯一标识符
    fn id(&self) -> &'static str;
    
    /// 技能展示名称
    fn name(&self) -> &'static str;
    
    /// 执行技能逻辑 (这里用 String 代替具体的 LLMProvider 注入，稍后对接实际 API)
    async fn execute(
        &self,
        context: SkillContext,
    ) -> anyhow::Result<SkillOutput>;
}
