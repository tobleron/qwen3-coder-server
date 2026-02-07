use std::fs;
use std::path::Path;
use anyhow::Result;

pub struct PromptManager {
    prompts_dir: String,
}

impl PromptManager {
    pub fn new(prompts_dir: String) -> Self {
        Self { prompts_dir }
    }

    pub fn list_prompts(&self) -> Result<Vec<String>> {
        let mut prompts = Vec::new();

        let dir = Path::new(&self.prompts_dir);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
            return Ok(prompts);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    prompts.push(name.to_string());
                }
            }
        }

        prompts.sort();
        Ok(prompts)
    }

    pub fn load_prompt(&self, name: &str) -> Result<String> {
        let path = Path::new(&self.prompts_dir).join(format!("{}.txt", name));
        Ok(fs::read_to_string(path)?)
    }

    #[allow(dead_code)]
    pub fn save_prompt(&self, name: &str, content: &str) -> Result<()> {
        fs::create_dir_all(&self.prompts_dir)?;
        let path = Path::new(&self.prompts_dir).join(format!("{}.txt", name));
        fs::write(path, content)?;
        Ok(())
    }
}
