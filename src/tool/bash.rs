use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;

#[derive(Deserialize, Serialize)]
pub struct Bash;

#[derive(Deserialize)]
pub struct BashArgs {
    pub command: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Bash command error")]
pub struct BashError;

impl Tool for Bash {
    const NAME: &'static str = "bash";
    type Error = BashError;
    type Args = BashArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "bash".to_string(),
            description: "Run a bash command".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(&args.command)
            .output()
            .await
            .map_err(|_| BashError)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stderr.is_empty() {
            Ok(format!("{}\n{}", stdout, stderr))
        } else {
            Ok(stdout.to_string())
        }
    }
}
