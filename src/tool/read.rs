use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct ReadFile;

#[derive(Deserialize)]
pub struct ReadFileArgs {
    pub path: String,
}

#[derive(Debug, thiserror::Error)]
#[error("File operation error")]
pub struct FileError;

impl Tool for ReadFile {
    const NAME: &'static str = "read";
    type Error = FileError;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "read".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let content = std::fs::read_to_string(&args.path)
            .map_err(|_| FileError)?;
        Ok(content)
    }
}