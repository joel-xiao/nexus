use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

pub struct PromptStore {
    registry: Handlebars<'static>,
    pub(crate) templates: HashMap<String, String>,
}

impl PromptStore {
    pub fn new() -> Self {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(false);

        Self {
            registry,
            templates: HashMap::new(),
        }
    }

    pub fn register_template(&mut self, name: &str, template: &str) -> anyhow::Result<()> {
        info!("Registering template: {}", name);
        self.registry.register_template_string(name, template)?;
        self.templates
            .insert(name.to_string(), template.to_string());
        Ok(())
    }

    pub fn render(&self, template_name: &str, variables: &Value) -> anyhow::Result<String> {
        self.render_with_template(
            template_name,
            &self
                .templates
                .get(template_name)
                .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))?,
            variables,
        )
    }

    pub fn render_with_template(
        &self,
        _name: &str,
        template: &str,
        variables: &Value,
    ) -> anyhow::Result<String> {
        let result = self.registry.render_template(template, &variables)?;
        Ok(result)
    }

    pub fn render_string(
        &mut self,
        template_str: &str,
        variables: &Value,
    ) -> anyhow::Result<String> {
        let result = self.registry.render_template(template_str, variables)?;
        Ok(result)
    }

    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    pub fn unregister_template(&mut self, name: &str) {
        self.templates.remove(name);
    }
}

impl Default for PromptStore {
    fn default() -> Self {
        Self::new()
    }
}
