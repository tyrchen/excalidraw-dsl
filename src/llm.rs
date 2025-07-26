// src/llm.rs
#[cfg(feature = "llm")]
use crate::error::{LLMError, Result};
#[cfg(feature = "llm")]
use crate::igr::IntermediateGraph;
#[cfg(feature = "llm")]
use petgraph::visit::{EdgeRef, IntoNodeReferences};
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
    cache: std::collections::HashMap<String, Vec<LayoutAdjustment>>,
    optimization_strategies: OptimizationStrategies,
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
#[derive(Debug, Deserialize, Clone)]
pub struct LayoutAdjustment {
    pub id: String,
    pub x_move: Option<f64>,
    pub y_move: Option<f64>,
    pub reason: Option<String>,
}

#[cfg(feature = "llm")]
#[derive(Debug, Clone)]
pub struct OptimizationStrategies {
    pub semantic_positioning: bool,
    pub visual_balance: bool,
    pub flow_optimization: bool,
    pub container_organization: bool,
    pub performance_mode: bool,
}

impl Default for OptimizationStrategies {
    fn default() -> Self {
        Self {
            semantic_positioning: true,
            visual_balance: true,
            flow_optimization: true,
            container_organization: true,
            performance_mode: false,
        }
    }
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize)]
pub struct EnhancedLayoutRequest {
    pub edsl_source: String,
    pub current_layout: Vec<NodePosition>,
    pub containers: Vec<ContainerInfo>,
    pub groups: Vec<GroupInfo>,
    pub edges: Vec<EdgeInfo>,
    pub diagram_type: Option<String>,
    pub optimization_focus: Vec<String>,
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize)]
pub struct GroupInfo {
    pub id: String,
    pub label: Option<String>,
    pub group_type: String,
    pub bounds: Option<(f64, f64, f64, f64)>,
    pub children: Vec<String>,
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize)]
pub struct EdgeInfo {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub edge_type: String,
}

#[cfg(feature = "llm")]
#[derive(Debug)]
pub struct OptimizationStats {
    pub cache_size: usize,
    pub enabled: bool,
    pub performance_mode: bool,
    pub strategies: OptimizationStrategies,
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
            cache: std::collections::HashMap::new(),
            optimization_strategies: OptimizationStrategies::default(),
        }
    }

    pub fn with_strategies(mut self, strategies: OptimizationStrategies) -> Self {
        self.optimization_strategies = strategies;
        self
    }

    pub fn enable_performance_mode(mut self) -> Self {
        self.optimization_strategies.performance_mode = true;
        self
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    pub fn optimize_layout(
        &mut self,
        igr: &mut IntermediateGraph,
        original_edsl: &str,
    ) -> Result<Vec<LayoutAdjustment>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        // Check cache first
        let cache_key = self.generate_cache_key(igr, original_edsl);
        if let Some(cached_adjustments) = self.cache.get(&cache_key) {
            self.apply_adjustments(igr, cached_adjustments)?;
            return Ok(cached_adjustments.clone());
        }

        let request = self.prepare_enhanced_request(igr, original_edsl);
        let prompt = self.build_enhanced_optimization_prompt(request);

        let rt = tokio::runtime::Runtime::new().map_err(|_| LLMError::ServiceUnavailable)?;

        let response = rt.block_on(self.client.query(&prompt))?;
        let adjustments = self.parse_adjustments(&response)?;

        // Cache the result
        self.cache.insert(cache_key, adjustments.clone());

        // Apply pre-validation
        let validated_adjustments = self.validate_adjustments(igr, &adjustments)?;
        self.apply_adjustments(igr, &validated_adjustments)?;

        Ok(validated_adjustments)
    }

    pub fn get_optimization_stats(&self) -> OptimizationStats {
        OptimizationStats {
            cache_size: self.cache.len(),
            enabled: self.enabled,
            performance_mode: self.optimization_strategies.performance_mode,
            strategies: self.optimization_strategies.clone(),
        }
    }

    fn generate_cache_key(&self, igr: &IntermediateGraph, edsl_source: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        edsl_source.hash(&mut hasher);

        // Hash node positions
        for (_, node) in igr.graph.node_references() {
            node.id.hash(&mut hasher);
            (node.x as i32).hash(&mut hasher);
            (node.y as i32).hash(&mut hasher);
        }

        format!("llm_opt_{}", hasher.finish())
    }

    fn validate_adjustments(
        &self,
        igr: &IntermediateGraph,
        adjustments: &[LayoutAdjustment],
    ) -> Result<Vec<LayoutAdjustment>> {
        let mut validated = Vec::new();

        for adjustment in adjustments {
            // Check if node exists
            if igr.get_node_by_id(&adjustment.id).is_none() {
                continue; // Skip invalid node references
            }

            // Validate movement bounds (prevent extreme movements)
            let mut valid_adjustment = adjustment.clone();

            if let Some(x_move) = adjustment.x_move {
                if x_move.abs() > 500.0 {
                    valid_adjustment.x_move = Some(x_move.signum() * 500.0);
                }
            }

            if let Some(y_move) = adjustment.y_move {
                if y_move.abs() > 500.0 {
                    valid_adjustment.y_move = Some(y_move.signum() * 500.0);
                }
            }

            validated.push(valid_adjustment);
        }

        Ok(validated)
    }

    fn prepare_enhanced_request(
        &self,
        igr: &IntermediateGraph,
        edsl_source: &str,
    ) -> EnhancedLayoutRequest {
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

        let groups: Vec<GroupInfo> = igr
            .groups
            .iter()
            .map(|group| GroupInfo {
                id: group.id.clone(),
                label: group.label.clone(),
                group_type: format!("{:?}", group.group_type),
                bounds: group.bounds.as_ref().map(|b| (b.x, b.y, b.width, b.height)),
                children: group
                    .children
                    .iter()
                    .filter_map(|&idx| igr.graph.node_weight(idx).map(|node| node.id.clone()))
                    .collect(),
            })
            .collect();

        let edges: Vec<EdgeInfo> = igr
            .graph
            .edge_references()
            .map(|edge_ref| {
                let from_node = &igr.graph[edge_ref.source()];
                let to_node = &igr.graph[edge_ref.target()];
                let edge_data = edge_ref.weight();

                EdgeInfo {
                    from: from_node.id.clone(),
                    to: to_node.id.clone(),
                    label: edge_data.label.clone(),
                    edge_type: format!("{:?}", edge_data.arrow_type),
                }
            })
            .collect();

        // Determine optimization focus based on strategies
        let mut optimization_focus = Vec::new();
        if self.optimization_strategies.semantic_positioning {
            optimization_focus.push("semantic_positioning".to_string());
        }
        if self.optimization_strategies.visual_balance {
            optimization_focus.push("visual_balance".to_string());
        }
        if self.optimization_strategies.flow_optimization {
            optimization_focus.push("flow_optimization".to_string());
        }
        if self.optimization_strategies.container_organization {
            optimization_focus.push("container_organization".to_string());
        }

        EnhancedLayoutRequest {
            edsl_source: edsl_source.to_string(),
            current_layout,
            containers,
            groups,
            edges,
            diagram_type: None, // Could be inferred from EDSL content in future
            optimization_focus,
        }
    }

    fn build_enhanced_optimization_prompt(&self, request: EnhancedLayoutRequest) -> String {
        let focus_areas = if request.optimization_focus.is_empty() {
            "all aspects".to_string()
        } else {
            request.optimization_focus.join(", ")
        };

        let mut prompt = format!(
            r#"
You are a professional diagram layout optimizer with expertise in semantic positioning and visual design. Your task is to analyze the provided EDSL source code and current layout, then suggest improvements.

## Original EDSL Source:
```edsl
{}
```

## Current Layout (nodes with positions):
```json
{}
```
"#,
            request.edsl_source,
            serde_json::to_string_pretty(&request.current_layout).unwrap()
        );

        // Add groups information if available
        if !request.groups.is_empty() {
            prompt.push_str(&format!(
                "\n## Groups and Containers:\n```json\n{}\n```\n",
                serde_json::to_string_pretty(&request.groups).unwrap()
            ));
        }

        // Add edge information for flow analysis
        if !request.edges.is_empty() {
            prompt.push_str(&format!(
                "\n## Edge Connections:\n```json\n{}\n```\n",
                serde_json::to_string_pretty(&request.edges).unwrap()
            ));
        }

        prompt.push_str(&format!(
            r#"
## Optimization Focus: {}

## Your Task:
Based on the EDSL semantics and current layout, evaluate improvements focusing on:
1. **Semantic Positioning**: Position elements based on their logical role (e.g., databases at bottom, users at top, services in middle)
2. **Visual Balance**: Create symmetrical and aesthetically pleasing arrangements
3. **Flow Optimization**: Ensure logical flow direction matches data/control flow
4. **Container Organization**: Optimize grouping and hierarchical layout
5. **Edge Routing**: Minimize edge crossings and improve readability

**Performance Mode**: {}

**IMPORTANT**: Respond ONLY with a JSON array of adjustment objects. Each object must have:
- `id`: node identifier (must match existing nodes)
- `x_move`: horizontal movement in pixels (optional, positive = right)
- `y_move`: vertical movement in pixels (optional, positive = down)
- `reason`: brief explanation for the change (optional but recommended)

Example: [{{"id": "database", "y_move": 100, "reason": "move database to bottom layer for semantic clarity"}}]

Constraints:
- Maximum movement: ±500 pixels per adjustment
- Only move nodes that exist in the current layout
- Consider group boundaries and container relationships
- If no improvements needed, return empty array: []
"#,
            focus_areas,
            if self.optimization_strategies.performance_mode { "ENABLED - Prefer fewer, high-impact adjustments" } else { "DISABLED - Detailed optimization allowed" }
        ));

        prompt
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
            model: "gpt-4o".to_string(), // Use more recent model
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a diagram layout optimization expert specializing in semantic positioning and visual design. Respond only with valid JSON arrays containing layout adjustments.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.1,
            max_tokens: 2000, // Increased for more detailed responses
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
