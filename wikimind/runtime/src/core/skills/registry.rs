use super::{Skill, SkillContext, SkillOutput};
use std::collections::HashMap;

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        self.skills.insert(skill.id().to_string(), skill);
    }

    pub fn get_skill(&self, id: &str) -> Option<&Box<dyn Skill>> {
        self.skills.get(id)
    }
    
    pub fn list_skills(&self) -> Vec<(&str, &str)> {
        self.skills.values().map(|s| (s.id(), s.name())).collect()
    }
}
