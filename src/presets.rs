// src/presets.rs
//! Predefined diagram presets for common use cases

use crate::ast::ArrowType;
use crate::fluent::DiagramBuilder;
use std::collections::HashMap;

/// Common diagram presets
pub struct DiagramPresets;

impl DiagramPresets {
    /// Create a basic client-server architecture diagram
    pub fn client_server() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("dagre")
            .node("client")
            .label("Client")
            .shape("rectangle")
            .background("#e3f2fd")
            .done()
            .node("server")
            .label("Server")
            .shape("rectangle")
            .background("#fff3e0")
            .done()
            .node("database")
            .label("Database")
            .shape("cylinder")
            .background("#f3e5f5")
            .done()
            .edge("client", "server")
            .label("HTTP Request")
            .done()
            .edge("server", "database")
            .label("Query")
            .done()
    }

    /// Create a microservices architecture diagram
    pub fn microservices() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("elk")
            .container("API Gateway")
            .color("#1976d2")
            .with_node("gateway", Some("Gateway"))
            .done()
            .container("Services")
            .with_node("auth", Some("Auth Service"))
            .with_node("user", Some("User Service"))
            .with_node("order", Some("Order Service"))
            .done()
            .container("Data Layer")
            .with_node("auth_db", Some("Auth DB"))
            .with_node("user_db", Some("User DB"))
            .with_node("order_db", Some("Order DB"))
            .done()
    }

    /// Create a basic flow chart
    pub fn flowchart() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("dagre")
            .node("start")
            .label("Start")
            .shape("ellipse")
            .background("#c8e6c9")
            .done()
            .node("process1")
            .label("Process Data")
            .shape("rectangle")
            .done()
            .node("decision")
            .label("Valid?")
            .shape("diamond")
            .background("#fff9c4")
            .done()
            .node("end")
            .label("End")
            .shape("ellipse")
            .background("#ffcdd2")
            .done()
    }

    /// Create a state machine diagram
    pub fn state_machine() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("force")
            .node("idle")
            .label("Idle")
            .shape("ellipse")
            .done()
            .node("processing")
            .label("Processing")
            .shape("ellipse")
            .done()
            .node("completed")
            .label("Completed")
            .shape("ellipse")
            .background("#a5d6a7")
            .done()
            .node("error")
            .label("Error")
            .shape("ellipse")
            .background("#ef9a9a")
            .done()
    }

    /// Create a network topology diagram
    pub fn network_topology() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("force")
            .node("internet")
            .label("Internet")
            .shape("ellipse")
            .background("#90caf9")
            .done()
            .node("firewall")
            .label("Firewall")
            .shape("rectangle")
            .background("#ffab91")
            .done()
            .node("load_balancer")
            .label("Load Balancer")
            .shape("rectangle")
            .background("#ce93d8")
            .done()
            .container("DMZ")
            .color("#ff5722")
            .with_node("web1", Some("Web Server 1"))
            .with_node("web2", Some("Web Server 2"))
            .done()
            .container("Internal Network")
            .color("#4caf50")
            .with_node("app1", Some("App Server 1"))
            .with_node("app2", Some("App Server 2"))
            .with_node("db", Some("Database"))
            .done()
    }

    /// Create a CI/CD pipeline diagram
    pub fn cicd_pipeline() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("dagre")
            .node("code")
            .label("Source Code")
            .shape("rectangle")
            .background("#e8f5e9")
            .done()
            .node("build")
            .label("Build")
            .shape("rectangle")
            .done()
            .node("test")
            .label("Test")
            .shape("rectangle")
            .done()
            .node("deploy_staging")
            .label("Deploy to Staging")
            .shape("rectangle")
            .background("#fff3e0")
            .done()
            .node("deploy_prod")
            .label("Deploy to Production")
            .shape("rectangle")
            .background("#ffebee")
            .done()
            .edge("code", "build")
            .done()
            .edge("build", "test")
            .done()
            .edge("test", "deploy_staging")
            .done()
            .edge("deploy_staging", "deploy_prod")
            .label("Manual Approval")
            .style("dashed")
            .done()
    }

    /// Create a class diagram (UML-style)
    pub fn class_diagram() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("dagre")
            .node("animal")
            .label("Animal\n+name: String\n+age: int\n+makeSound()")
            .shape("rectangle")
            .done()
            .node("dog")
            .label("Dog\n+breed: String\n+bark()")
            .shape("rectangle")
            .done()
            .node("cat")
            .label("Cat\n+color: String\n+meow()")
            .shape("rectangle")
            .done()
            .edge("dog", "animal")
            .arrow_type(ArrowType::Line)
            .style("solid")
            .done()
            .edge("cat", "animal")
            .arrow_type(ArrowType::Line)
            .style("solid")
            .done()
    }

    /// Create a Kubernetes deployment diagram
    pub fn kubernetes() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("elk")
            .container("Kubernetes Cluster")
            .color("#326ce5")
            .done()
            .container("Namespace: production")
            .with_node("deployment", Some("Deployment"))
            .with_node("service", Some("Service"))
            .with_node("ingress", Some("Ingress"))
            .done()
            .container("Pods")
            .with_node("pod1", Some("Pod 1"))
            .with_node("pod2", Some("Pod 2"))
            .with_node("pod3", Some("Pod 3"))
            .done()
    }

    /// Create a data flow diagram
    pub fn data_flow() -> DiagramBuilder {
        DiagramBuilder::new()
            .with_layout("dagre")
            .node("source")
            .label("Data Source")
            .shape("cylinder")
            .done()
            .node("extract")
            .label("Extract")
            .shape("rectangle")
            .done()
            .node("transform")
            .label("Transform")
            .shape("rectangle")
            .done()
            .node("load")
            .label("Load")
            .shape("rectangle")
            .done()
            .node("warehouse")
            .label("Data Warehouse")
            .shape("cylinder")
            .done()
            .edge("source", "extract")
            .done()
            .edge("extract", "transform")
            .done()
            .edge("transform", "load")
            .done()
            .edge("load", "warehouse")
            .done()
    }
}

/// Preset themes for consistent styling
pub struct ThemePresets;

impl ThemePresets {
    /// Material Design color palette
    pub fn material_colors() -> HashMap<&'static str, &'static str> {
        let mut colors = HashMap::new();
        colors.insert("red", "#f44336");
        colors.insert("pink", "#e91e63");
        colors.insert("purple", "#9c27b0");
        colors.insert("deep_purple", "#673ab7");
        colors.insert("indigo", "#3f51b5");
        colors.insert("blue", "#2196f3");
        colors.insert("light_blue", "#03a9f4");
        colors.insert("cyan", "#00bcd4");
        colors.insert("teal", "#009688");
        colors.insert("green", "#4caf50");
        colors.insert("light_green", "#8bc34a");
        colors.insert("lime", "#cddc39");
        colors.insert("yellow", "#ffeb3b");
        colors.insert("amber", "#ffc107");
        colors.insert("orange", "#ff9800");
        colors.insert("deep_orange", "#ff5722");
        colors.insert("brown", "#795548");
        colors.insert("grey", "#9e9e9e");
        colors.insert("blue_grey", "#607d8b");
        colors
    }

    /// Corporate/professional color scheme
    pub fn corporate_theme() -> HashMap<&'static str, &'static str> {
        let mut colors = HashMap::new();
        colors.insert("primary", "#1976d2");
        colors.insert("secondary", "#424242");
        colors.insert("accent", "#ff6f00");
        colors.insert("success", "#388e3c");
        colors.insert("warning", "#f57c00");
        colors.insert("error", "#d32f2f");
        colors.insert("info", "#0288d1");
        colors.insert("light", "#f5f5f5");
        colors.insert("dark", "#212121");
        colors
    }

    /// Pastel color scheme
    pub fn pastel_theme() -> HashMap<&'static str, &'static str> {
        let mut colors = HashMap::new();
        colors.insert("pink", "#ffcdd2");
        colors.insert("purple", "#e1bee7");
        colors.insert("blue", "#bbdefb");
        colors.insert("cyan", "#b2ebf2");
        colors.insert("green", "#c8e6c9");
        colors.insert("yellow", "#fff9c4");
        colors.insert("orange", "#ffe0b2");
        colors.insert("grey", "#f5f5f5");
        colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_server_preset() {
        let ast = DiagramPresets::client_server().build_ast();
        assert_eq!(ast.nodes.len(), 3);
        assert_eq!(ast.edges.len(), 2);
    }

    #[test]
    fn test_material_colors() {
        let colors = ThemePresets::material_colors();
        assert_eq!(colors.get("blue"), Some(&"#2196f3"));
    }
}
