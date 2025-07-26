// src/llm.rs
#[cfg(feature = "llm")]
use crate::error::{LLMError, Result};
#[cfg(feature = "llm")]
use crate::igr::IntermediateGraph;
#[cfg(feature = "llm")]
use petgraph::visit::IntoNodeReferences;
#[cfg(feature = "llm")]
use reqwest::Client;
#[cfg(feature = "llm")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "llm")]
use std::time::Duration;

#[cfg(feature = "llm")]
pub struct LLMLayoutOptimizer {
    client: LLMClient,
    enabled: bool,
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize)]
pub struct LayoutOptimizationRequest {
    pub edsl_source: String,
    pub current_layout: Vec<NodePosition>,
    pub containers: Vec<ContainerInfo>,
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize, Deserialize)]
pub struct NodePosition {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub label: String,
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub id: Option<String>,
    pub label: Option<String>,
    pub bounds: Option<(f64, f64, f64, f64)>, // x, y, width, height
}

#[cfg(feature = "llm")]
#[derive(Debug, Deserialize)]
pub struct LayoutAdjustment {
    pub id: String,
    pub x_move: Option<f64>,
    pub y_move: Option<f64>,
    pub reason: Option<String>,
}

#[cfg(feature = "llm")]
struct LLMClient {
    client: Client,
    api_key: String,
    endpoint: String,
}

#[cfg(feature = "llm")]
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[cfg(feature = "llm")]
#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[cfg(feature = "llm")]
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[cfg(feature = "llm")]
#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[cfg(feature = "llm")]
impl LLMLayoutOptimizer {
    pub fn new(api_key: String) -> Self {
        Self {
            client: LLMClient::new(api_key),
            enabled: true,
        }
    }

    pub fn optimize_layout(
        &self,
        igr: &mut IntermediateGraph,
        original_edsl: &str,
    ) -> Result<Vec<LayoutAdjustment>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        let request = self.prepare_request(igr, original_edsl);
        let prompt = self.build_optimization_prompt(request);

        let rt = tokio::runtime::Runtime::new().map_err(|_| LLMError::ServiceUnavailable)?;

        let response = rt.block_on(self.client.query(&prompt))?;
        let adjustments = self.parse_adjustments(&response)?;

        self.apply_adjustments(igr, &adjustments)?;

        Ok(adjustments)
    }

    fn prepare_request(
        &self,
        igr: &IntermediateGraph,
        edsl_source: &str,
    ) -> LayoutOptimizationRequest {
        let current_layout: Vec<NodePosition> = igr
            .graph
            .node_references()
            .map(|(_, node)| NodePosition {
                id: node.id.clone(),
                x: node.x,
                y: node.y,
                width: node.width,
                height: node.height,
                label: node.label.clone(),
            })
            .collect();

        let containers: Vec<ContainerInfo> = igr
            .containers
            .iter()
            .map(|container| ContainerInfo {
                id: container.id.clone(),
                label: container.label.clone(),
                bounds: container
                    .bounds
                    .as_ref()
                    .map(|b| (b.x, b.y, b.width, b.height)),
            })
            .collect();

        LayoutOptimizationRequest {
            edsl_source: edsl_source.to_string(),
            current_layout,
            containers,
        }
    }

    fn build_optimization_prompt(&self, request: LayoutOptimizationRequest) -> String {
        format!(
            r#"
You are a professional diagram layout optimizer. Your task is to analyze the provided EDSL source code and current layout positions, then suggest improvements to enhance semantic clarity and visual appeal.

## Original EDSL Source:
```edsl
{}
```

## Current Layout (nodes with positions):
```json
{}
```

## Your Task:
Based on the EDSL semantics, evaluate if the current layout can be improved. Consider:
1. Semantic positioning (e.g., databases typically at bottom, users at top)
2. Visual balance and symmetry
3. Logical flow direction
4. Container organization

**IMPORTANT**: Respond ONLY with a JSON array of adjustment objects. Each object must have:
- `id`: node identifier
- `x_move`: horizontal movement (optional)
- `y_move`: vertical movement (optional)
- `reason`: brief explanation (optional)

Example: [{{"id": "database", "y_move": 50, "reason": "move database to bottom layer"}}]

If no improvements needed, return empty array: []
        "#,
            request.edsl_source,
            serde_json::to_string_pretty(&request.current_layout).unwrap()
        )
    }

    fn parse_adjustments(&self, response: &str) -> Result<Vec<LayoutAdjustment>> {
        // Extract JSON from response (handle potential markdown formatting)
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        serde_json::from_str(json_str).map_err(|e| LLMError::InvalidResponse(e.to_string()).into())
    }

    fn apply_adjustments(
        &self,
        igr: &mut IntermediateGraph,
        adjustments: &[LayoutAdjustment],
    ) -> Result<()> {
        for adjustment in adjustments {
            if let Some((_, node)) = igr.get_node_mut_by_id(&adjustment.id) {
                if let Some(x_move) = adjustment.x_move {
                    node.x += x_move;
                }
                if let Some(y_move) = adjustment.y_move {
                    node.y += y_move;
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "llm")]
impl LLMClient {
    fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            api_key,
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
        }
    }

    async fn query(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a diagram layout optimization expert. Respond only with valid JSON.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.1,
            max_tokens: 1000,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LLMError::Timeout
                } else {
                    LLMError::Http(e)
                }
            })?;

        if response.status() == 401 {
            return Err(LLMError::AuthenticationFailed.into());
        } else if response.status() == 429 {
            return Err(LLMError::QuotaExceeded.into());
        } else if !response.status().is_success() {
            return Err(LLMError::ServiceUnavailable.into());
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

        chat_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| LLMError::InvalidResponse("No response choices".to_string()).into())
    }
}

#[cfg(not(feature = "llm"))]
pub struct LLMLayoutOptimizer;

#[cfg(not(feature = "llm"))]
impl LLMLayoutOptimizer {
    pub fn new(_api_key: String) -> Self {
        Self
    }

    pub fn optimize_layout(
        &self,
        _igr: &mut IntermediateGraph,
        _original_edsl: &str,
    ) -> Result<Vec<()>> {
        Ok(vec![])
    }
}
