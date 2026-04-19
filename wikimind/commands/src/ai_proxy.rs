use tauri::AppHandle;
use wikimind_runtime::core::skills::registry::SkillRegistry;
use wikimind_runtime::core::skills::builtin::{OrganizeToWikiSkill, AutoAgentSkill};
use wikimind_runtime::core::skills::{SkillContext, SkillOutput};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Serialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub async fn list_skills() -> Result<Vec<SkillInfo>, String> {
    let mut registry = SkillRegistry::new();
    registry.register(Box::new(OrganizeToWikiSkill));
    registry.register(Box::new(AutoAgentSkill));
    
    let skills = registry.list_skills()
        .into_iter()
        .map(|(id, name)| SkillInfo {
            id: id.to_string(),
            name: name.to_string(),
        })
        .collect();
        
    Ok(skills)
}

#[derive(Deserialize)]
pub struct ExecuteSkillPayload {
    pub skill_id: String,
    pub original_content: String,
    pub instruction: Option<String>,
    pub source_path: Option<String>,
    pub target_wiki: Option<String>,
}

#[tauri::command]
pub async fn execute_skill(payload: ExecuteSkillPayload) -> Result<SkillOutput, String> {
    info!("Executing skill: {}", payload.skill_id);
    
    let mut registry = SkillRegistry::new();
    registry.register(Box::new(OrganizeToWikiSkill));
    registry.register(Box::new(AutoAgentSkill));
    
    let skill = registry.get_skill(&payload.skill_id)
        .ok_or_else(|| format!("Skill not found: {}", payload.skill_id))?;
        
    let context = SkillContext {
        instruction: payload.instruction,
        original_content: payload.original_content,
        source_path: payload.source_path,
        target_wiki: payload.target_wiki,
    };
    
    match skill.execute(context).await {
        Ok(output) => Ok(output),
        Err(e) => {
            error!("Skill execution failed: {}", e);
            Err(e.to_string())
        }
    }
}
