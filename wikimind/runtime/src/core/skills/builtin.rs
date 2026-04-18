use super::{Skill, SkillContext, SkillOutput};
use crate::infra::diff::generate_diff;
use async_trait::async_trait;
use serde_json::json;

/// Organize To Wiki Skill (整理到 Wiki)
/// Inspired by llm-wiki-compiler's page generation prompt.
pub struct OrganizeToWikiSkill;

#[async_trait]
impl Skill for OrganizeToWikiSkill {
    fn id(&self) -> &'static str {
        "organize_to_wiki"
    }

    fn name(&self) -> &'static str {
        "整理到 Wiki"
    }

    async fn execute(&self, context: SkillContext) -> anyhow::Result<SkillOutput> {
        // 构建提示词
        let mut prompt = String::from("You are a wiki author. Write a clear, well-structured markdown page.\n");
        prompt.push_str("Draw facts only from the provided source material.\n");
        prompt.push_str("Include a ## Sources section at the end listing the source document.\n");
        prompt.push_str("Suggest [[wikilinks]] to related concepts where appropriate.\n\n");
        
        if let Some(inst) = &context.instruction {
            prompt.push_str(&format!("User Instruction: {}\n\n", inst));
        }
        
        prompt.push_str("--- SOURCE MATERIAL ---\n\n");
        prompt.push_str(&context.original_content);

        // TODO: 调用真实的 LLMProvider
        // 这里模拟 LLM 返回的提议内容
        let proposed_content = format!("{}\n\n## Sources\n- Extracted from raw notes.", context.original_content);
        
        // 使用 infra::diff 生成差异
        let diff = generate_diff(&context.original_content, &proposed_content);

        Ok(SkillOutput {
            proposed_content,
            diff: Some(diff),
            title_suggestion: Some("整理后的页面".to_string()),
            meta: Some(json!({
                "source": context.source_path,
                "created_by": "ai"
            })),
        })
    }
}

/// Auto Agent Skill (AI 自动整理)
/// Inspired by mnemovault's ingest prompt.
pub struct AutoAgentSkill;

#[async_trait]
impl Skill for AutoAgentSkill {
    fn id(&self) -> &'static str {
        "auto_agent_extract"
    }

    fn name(&self) -> &'static str {
        "AI 自动整理"
    }

    async fn execute(&self, context: SkillContext) -> anyhow::Result<SkillOutput> {
        // 构建提示词
        let mut prompt = String::from("You are the knowledge compiler for an LLM wiki.\n");
        prompt.push_str("Your job is to read a new raw source, extract distinct concepts, and make the wiki more coherent and better linked.\n");
        prompt.push_str("Assign an evidence_type to each claim: EXTRACTED | INFERRED.\n");
        prompt.push_str("Use [[wikilink]] syntax for all cross-references.\n\n");
        
        prompt.push_str("--- SOURCE MATERIAL ---\n\n");
        prompt.push_str(&context.original_content);

        // TODO: 调用真实的 LLMProvider
        let proposed_content = format!("# 自动整理结果\n\n{}", context.original_content);
        let diff = generate_diff(&context.original_content, &proposed_content);

        Ok(SkillOutput {
            proposed_content,
            diff: Some(diff),
            title_suggestion: Some("自动提取的知识点".to_string()),
            meta: Some(json!({
                "source": context.source_path,
                "created_by": "ai",
                "wiki_type": "agent-wiki"
            })),
        })
    }
}
