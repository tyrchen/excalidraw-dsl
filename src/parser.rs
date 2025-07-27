// src/parser.rs
use crate::ast::*;
use crate::error::{ParseError, Result};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use uuid::Uuid;

// Security limits to prevent DoS attacks
const MAX_INPUT_SIZE: usize = 1_000_000; // 1MB
const MAX_NODES: usize = 1000;
const MAX_EDGES: usize = 5000;
const MAX_CONTAINERS: usize = 100;

#[derive(Parser)]
#[grammar = "edsl.pest"]
pub struct EDSLParser;

pub fn parse_edsl(input: &str) -> Result<ParsedDocument> {
    // Validate input size
    if input.len() > MAX_INPUT_SIZE {
        return Err(ParseError::ValidationError(format!(
            "Input size exceeds maximum allowed size of {MAX_INPUT_SIZE} bytes"
        ))
        .into());
    }

    let pairs =
        EDSLParser::parse(Rule::file, input).map_err(|e| ParseError::PestError(Box::new(e)))?;

    build_document(pairs)
}

fn build_document(pairs: pest::iterators::Pairs<Rule>) -> Result<ParsedDocument> {
    let mut config = GlobalConfig::default();
    let mut component_types = HashMap::new();
    let mut templates = HashMap::new();
    let mut diagram = None;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut containers = Vec::new();
    let mut groups = Vec::new();
    let mut connections = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::file {
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::config => {
                        config = parse_config(inner_pair)?;
                    }
                    Rule::statement => {
                        for stmt_pair in inner_pair.into_inner() {
                            match stmt_pair.as_rule() {
                                Rule::component_type_def => {
                                    let comp_type = parse_component_type(stmt_pair)?;
                                    component_types.insert(comp_type.name.clone(), comp_type);
                                }
                                Rule::node_def => {
                                    nodes.push(parse_node_definition(stmt_pair)?);
                                }
                                Rule::edge_def => {
                                    let parsed_edges = parse_edge_definition(stmt_pair)?;
                                    edges.extend(parsed_edges);
                                }
                                Rule::container_def => {
                                    containers.push(parse_container_definition(stmt_pair)?);
                                }
                                Rule::group_def => {
                                    groups.push(parse_group_definition(stmt_pair)?);
                                }
                                Rule::connection_def => {
                                    connections.push(parse_connection(stmt_pair)?);
                                }
                                Rule::connections_def => {
                                    let conns = parse_connections(stmt_pair)?;
                                    connections.extend(conns);
                                }
                                Rule::template_def => {
                                    let template = parse_template_definition(stmt_pair)?;
                                    templates.insert(template.name.clone(), template);
                                }
                                Rule::diagram_def => {
                                    if diagram.is_some() {
                                        return Err(ParseError::ValidationError(
                                            "Only one diagram definition is allowed per file"
                                                .to_string(),
                                        )
                                        .into());
                                    }
                                    diagram = Some(parse_diagram_definition(stmt_pair)?);
                                }
                                _ => {
                                    log::warn!("Unknown statement rule: {:?}", stmt_pair.as_rule());
                                }
                            }
                        }
                    }
                    Rule::EOI => break,
                    _ => {}
                }
            }
        }
    }

    // Validate complexity limits
    if nodes.len() > MAX_NODES {
        return Err(ParseError::ValidationError(format!(
            "Number of nodes ({}) exceeds maximum allowed ({})",
            nodes.len(),
            MAX_NODES
        ))
        .into());
    }

    if edges.len() > MAX_EDGES {
        return Err(ParseError::ValidationError(format!(
            "Number of edges ({}) exceeds maximum allowed ({})",
            edges.len(),
            MAX_EDGES
        ))
        .into());
    }

    if containers.len() > MAX_CONTAINERS {
        return Err(ParseError::ValidationError(format!(
            "Number of containers ({}) exceeds maximum allowed ({})",
            containers.len(),
            MAX_CONTAINERS
        ))
        .into());
    }

    Ok(ParsedDocument {
        config,
        component_types,
        templates,
        diagram,
        nodes,
        edges,
        containers,
        groups,
        connections,
    })
}

fn parse_config(pair: pest::iterators::Pair<Rule>) -> Result<GlobalConfig> {
    let yaml_content = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::yaml_content)
        .map(|p| p.as_str())
        .unwrap_or("");

    if yaml_content.trim().is_empty() {
        return Ok(GlobalConfig::default());
    }

    // Simply trim each line and reconstruct
    let clean_lines: Vec<String> = yaml_content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let clean_yaml = clean_lines.join("\n");

    serde_yaml::from_str(&clean_yaml).map_err(|e| ParseError::InvalidConfig(e.to_string()).into())
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Statement> {
    let inner = pair.into_inner().next().ok_or_else(|| ParseError::Syntax {
        line: 0,
        message: "Expected statement content".to_string(),
    })?;

    match inner.as_rule() {
        Rule::node_def => Ok(Statement::Node(parse_node_definition(inner)?)),
        Rule::edge_def => {
            // Edge chains need special handling since they expand to multiple edges
            let edges = parse_edge_definition(inner)?;
            if edges.len() == 1 {
                Ok(Statement::Edge(edges.into_iter().next().unwrap()))
            } else {
                // For now, return the first edge and log a warning
                // A more complete solution would require changing Statement to support multiple edges
                log::warn!("Edge chain in container/group context - only first edge will be used");
                Ok(Statement::Edge(edges.into_iter().next().unwrap()))
            }
        }
        Rule::container_def => Ok(Statement::Container(parse_container_definition(inner)?)),
        Rule::group_def => Ok(Statement::Group(parse_group_definition(inner)?)),
        _ => Err(ParseError::Syntax {
            line: 0,
            message: format!("Unexpected rule in statement: {:?}", inner.as_rule()),
        }
        .into()),
    }
}

fn parse_component_type(pair: pest::iterators::Pair<Rule>) -> Result<ComponentTypeDefinition> {
    let mut name = String::new();
    let mut shape = None;
    let mut style = StyleDefinition {
        fill: None,
        stroke_color: None,
        stroke_width: None,
        stroke_style: None,
        rounded: None,
        fill_style: None,
        roughness: None,
        font_size: None,
        font: None,
    };

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::id => {
                name = inner_pair.as_str().to_string();
            }
            Rule::component_type_style => {
                for style_pair in inner_pair.into_inner() {
                    match style_pair.as_rule() {
                        Rule::shape_type => {
                            shape = Some(style_pair.as_str().to_string());
                        }
                        Rule::style_block => {
                            let attrs = parse_style_block(style_pair)?;
                            // Convert attributes to style fields
                            if let Some(AttributeValue::String(s)) = attrs.get("fill") {
                                style.fill = Some(s.clone());
                            }
                            if let Some(AttributeValue::String(s)) = attrs.get("strokeColor") {
                                style.stroke_color = Some(s.clone());
                            }
                            if let Some(AttributeValue::Number(n)) = attrs.get("strokeWidth") {
                                style.stroke_width = Some(*n);
                            }
                            if let Some(AttributeValue::String(s)) = attrs.get("strokeStyle") {
                                style.stroke_style = s.parse().ok();
                            }
                            if let Some(AttributeValue::Number(n)) = attrs.get("rounded") {
                                style.rounded = Some(*n);
                            }
                            if let Some(AttributeValue::String(s)) = attrs.get("fillStyle") {
                                style.fill_style = s.parse().ok();
                            }
                            if let Some(AttributeValue::Number(n)) = attrs.get("roughness") {
                                style.roughness = Some(*n as u8);
                            }
                            if let Some(AttributeValue::Number(n)) = attrs.get("fontSize") {
                                style.font_size = Some(*n);
                            }
                            if let Some(AttributeValue::String(s)) = attrs.get("font") {
                                style.font = Some(s.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ComponentTypeDefinition { name, shape, style })
}

fn parse_node_definition(pair: pest::iterators::Pair<Rule>) -> Result<NodeDefinition> {
    let mut id = String::new();
    let mut label = None;
    let mut component_type = None;
    let mut attributes = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::id => {
                id = inner_pair.as_str().to_string();
            }
            Rule::label => {
                let label_text = inner_pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::label_text)
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_else(|| id.clone());
                label = Some(label_text);
            }
            Rule::type_ref => {
                for type_pair in inner_pair.into_inner() {
                    if type_pair.as_rule() == Rule::id {
                        component_type = Some(type_pair.as_str().to_string());
                    }
                }
            }
            Rule::style_block => {
                attributes = parse_style_block(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(NodeDefinition {
        id,
        label,
        component_type,
        attributes,
    })
}

fn parse_edge_definition(pair: pest::iterators::Pair<Rule>) -> Result<Vec<EdgeDefinition>> {
    let inner = pair.into_inner().next().ok_or_else(|| ParseError::Syntax {
        line: 0,
        message: "Expected edge content".to_string(),
    })?;

    match inner.as_rule() {
        Rule::single_edge => Ok(vec![parse_single_edge(inner)?]),
        Rule::edge_chain => parse_edge_chain(inner),
        _ => Err(ParseError::Syntax {
            line: 0,
            message: format!("Unexpected rule in edge definition: {:?}", inner.as_rule()),
        }
        .into()),
    }
}

fn parse_node_ref(pair: pest::iterators::Pair<Rule>) -> Result<(String, Option<String>)> {
    let mut id = String::new();
    let mut label = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::id => {
                id = inner_pair.as_str().to_string();
            }
            Rule::label => {
                let label_text = inner_pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::label_text)
                    .map(|p| p.as_str().to_string());
                label = label_text;
            }
            _ => {}
        }
    }

    Ok((id, label))
}

fn parse_single_edge(pair: pest::iterators::Pair<Rule>) -> Result<EdgeDefinition> {
    let mut from = String::new();
    let mut to = String::new();
    let mut arrow_type = ArrowType::SingleArrow;
    let mut label = None;
    let mut attributes = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::node_ref => {
                let (node_id, _node_label) = parse_node_ref(inner_pair)?;
                if from.is_empty() {
                    from = node_id;
                } else {
                    to = node_id;
                }
            }
            Rule::arrow => {
                arrow_type = inner_pair
                    .as_str()
                    .parse()
                    .unwrap_or(ArrowType::SingleArrow);
            }
            Rule::edge_label => {
                for label_part in inner_pair.into_inner() {
                    if label_part.as_rule() == Rule::edge_label_content {
                        let content = label_part.as_str();
                        if content.starts_with('"') && content.ends_with('"') {
                            label = Some(parse_string_literal(content)?);
                        } else {
                            label = Some(content.trim().to_string());
                        }
                    }
                }
            }
            Rule::style_block => {
                attributes = parse_style_block(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(EdgeDefinition {
        from,
        to,
        label,
        arrow_type,
        attributes,
        style: None,
    })
}

fn parse_edge_chain(pair: pest::iterators::Pair<Rule>) -> Result<Vec<EdgeDefinition>> {
    // Parse edge chain and expand into multiple edges
    let mut ids = Vec::new();
    let mut arrow_type = ArrowType::SingleArrow;
    let mut label = None;
    let mut attributes = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::node_ref => {
                let (node_id, _node_label) = parse_node_ref(inner_pair)?;
                ids.push(node_id);
            }
            Rule::arrow => {
                arrow_type = inner_pair
                    .as_str()
                    .parse()
                    .unwrap_or(ArrowType::SingleArrow);
            }
            Rule::edge_label => {
                for label_part in inner_pair.into_inner() {
                    if label_part.as_rule() == Rule::edge_label_content {
                        let content = label_part.as_str();
                        if content.starts_with('"') && content.ends_with('"') {
                            label = Some(parse_string_literal(content)?);
                        } else {
                            label = Some(content.trim().to_string());
                        }
                    }
                }
            }
            Rule::style_block => {
                attributes = parse_style_block(inner_pair)?;
            }
            _ => {}
        }
    }

    if ids.len() >= 2 {
        // Expand the edge chain into individual edges
        let mut edges = Vec::new();
        for i in 0..(ids.len() - 1) {
            edges.push(EdgeDefinition {
                from: ids[i].clone(),
                to: ids[i + 1].clone(),
                label: label.clone(),
                arrow_type,
                attributes: attributes.clone(),
                style: None,
            });
        }
        Ok(edges)
    } else {
        Err(ParseError::Syntax {
            line: 0,
            message: "Edge chain requires at least two nodes".to_string(),
        }
        .into())
    }
}

fn parse_group_definition(pair: pest::iterators::Pair<Rule>) -> Result<GroupDefinition> {
    let mut id = None;
    let mut label = None;
    let mut group_type = GroupType::BasicGroup;
    let mut attributes = HashMap::new();
    let mut internal_statements = Vec::new();
    let mut children = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::group_type => {
                group_type = parse_group_type(inner_pair)?;
            }
            Rule::string_literal => {
                if label.is_none() {
                    label = Some(parse_string_literal(inner_pair.as_str())?);
                }
            }
            Rule::id => {
                id = Some(inner_pair.as_str().to_string());
            }
            Rule::group_style => {
                let style_block = inner_pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::style_block)
                    .ok_or_else(|| ParseError::Syntax {
                        line: 0,
                        message: "Expected style block in group style".to_string(),
                    })?;
                attributes = parse_style_block(style_block)?;
            }
            Rule::statement => {
                // Check what kind of statement this is
                let stmt_inner = inner_pair.clone().into_inner().next().unwrap();
                match stmt_inner.as_rule() {
                    Rule::edge_def => {
                        // Handle edge definitions specially to support chains
                        let edges = parse_edge_definition(stmt_inner)?;
                        for edge in edges {
                            // Track children
                            if !children.contains(&edge.from) {
                                children.push(edge.from.clone());
                            }
                            if !children.contains(&edge.to) {
                                children.push(edge.to.clone());
                            }
                            internal_statements.push(Statement::Edge(edge));
                        }
                    }
                    _ => {
                        // For non-edge statements, use normal parsing
                        let statement = parse_statement(inner_pair)?;

                        match &statement {
                            Statement::Node(node) => {
                                children.push(node.id.clone());
                            }
                            Statement::Edge(edge) => {
                                if !children.contains(&edge.from) {
                                    children.push(edge.from.clone());
                                }
                                if !children.contains(&edge.to) {
                                    children.push(edge.to.clone());
                                }
                            }
                            _ => {}
                        }

                        internal_statements.push(statement);
                    }
                }
            }
            _ => {}
        }
    }

    // Generate ID from label if not provided
    let final_id = id.unwrap_or_else(|| {
        label
            .as_ref()
            .map(|l| l.to_lowercase().replace(' ', "_"))
            .unwrap_or_else(|| format!("group_{}", Uuid::new_v4()))
    });

    Ok(GroupDefinition {
        id: final_id,
        label,
        group_type,
        children,
        attributes,
        internal_statements,
    })
}

fn parse_group_type(pair: pest::iterators::Pair<Rule>) -> Result<GroupType> {
    let type_str = pair.as_str();
    match type_str {
        "group" => Ok(GroupType::BasicGroup),
        "flow" => Ok(GroupType::FlowGroup),
        _ => {
            // Check if it's a semantic group type
            for inner_pair in pair.into_inner() {
                if inner_pair.as_rule() == Rule::semantic_group_type {
                    return Ok(GroupType::SemanticGroup(inner_pair.as_str().to_string()));
                }
            }
            Ok(GroupType::SemanticGroup(type_str.to_string()))
        }
    }
}

fn parse_container_definition(pair: pest::iterators::Pair<Rule>) -> Result<ContainerDefinition> {
    let mut id = None;
    let mut label = None;
    let mut attributes = HashMap::new();
    let mut internal_statements = Vec::new();
    let mut children = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::string_literal => {
                if label.is_none() {
                    label = Some(parse_string_literal(inner_pair.as_str())?);
                }
            }
            Rule::id => {
                id = Some(inner_pair.as_str().to_string());
            }
            Rule::container_style => {
                let style_block = inner_pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::style_block)
                    .ok_or_else(|| ParseError::Syntax {
                        line: 0,
                        message: "Expected style block in container style".to_string(),
                    })?;
                attributes = parse_style_block(style_block)?;
            }
            Rule::statement => {
                // Check what kind of statement this is
                let stmt_inner = inner_pair.clone().into_inner().next().unwrap();
                match stmt_inner.as_rule() {
                    Rule::edge_def => {
                        // Handle edge definitions specially to support chains
                        let edges = parse_edge_definition(stmt_inner)?;
                        for edge in edges {
                            // Track children
                            if !children.contains(&edge.from) {
                                children.push(edge.from.clone());
                            }
                            if !children.contains(&edge.to) {
                                children.push(edge.to.clone());
                            }
                            internal_statements.push(Statement::Edge(edge));
                        }
                    }
                    _ => {
                        // For non-edge statements, use normal parsing
                        let statement = parse_statement(inner_pair)?;

                        match &statement {
                            Statement::Node(node) => {
                                children.push(node.id.clone());
                            }
                            Statement::Edge(edge) => {
                                if !children.contains(&edge.from) {
                                    children.push(edge.from.clone());
                                }
                                if !children.contains(&edge.to) {
                                    children.push(edge.to.clone());
                                }
                            }
                            _ => {}
                        }

                        internal_statements.push(statement);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ContainerDefinition {
        id,
        label,
        children,
        attributes,
        internal_statements,
    })
}

fn parse_style_block(pair: pest::iterators::Pair<Rule>) -> Result<HashMap<String, AttributeValue>> {
    let mut attributes = HashMap::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::attribute {
            let mut key = String::new();
            let mut value = None;

            for attr_pair in inner_pair.into_inner() {
                match attr_pair.as_rule() {
                    Rule::property_name => {
                        key = attr_pair.as_str().to_string();
                    }
                    Rule::property_value => {
                        value = Some(parse_property_value(attr_pair)?);
                    }
                    _ => {}
                }
            }

            if let Some(val) = value {
                attributes.insert(key, val);
            }
        }
    }

    Ok(attributes)
}

fn parse_property_value(pair: pest::iterators::Pair<Rule>) -> Result<AttributeValue> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::string_literal => Ok(AttributeValue::String(parse_string_literal(
            inner.as_str(),
        )?)),
        Rule::number => {
            let num_str = inner.as_str();
            let num = num_str.parse::<f64>().map_err(|_| ParseError::Syntax {
                line: 0,
                message: format!("Invalid number: {num_str}"),
            })?;
            Ok(AttributeValue::Number(num))
        }
        Rule::color => Ok(AttributeValue::Color(inner.as_str().to_string())),
        Rule::boolean => {
            let bool_val = inner.as_str() == "true";
            Ok(AttributeValue::Boolean(bool_val))
        }
        Rule::identifier => Ok(AttributeValue::String(inner.as_str().to_string())),
        _ => unreachable!(),
    }
}

fn parse_string_literal(s: &str) -> Result<String> {
    // Remove surrounding quotes
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Ok(s.to_string())
    }
}

fn parse_connection(pair: pest::iterators::Pair<Rule>) -> Result<ConnectionDefinition> {
    let mut from = String::new();
    let mut to = String::new();
    let mut style = EdgeStyleDefinition {
        edge_type: None,
        label: None,
        label_position: None,
        routing: None,
        color: None,
        width: None,
        stroke_style: None,
    };

    // Based on the grammar, the pairs come in order: from string, to string, style block
    let mut inner_pairs = pair.into_inner();

    // First string literal is "from"
    if let Some(from_pair) = inner_pairs.next() {
        if from_pair.as_rule() == Rule::string_literal {
            from = from_pair.as_str().trim_matches('"').to_string();
        }
    }

    // Second string literal is "to"
    if let Some(to_pair) = inner_pairs.next() {
        if to_pair.as_rule() == Rule::string_literal {
            to = to_pair.as_str().trim_matches('"').to_string();
        }
    }

    // Third is the style block
    if let Some(style_pair) = inner_pairs.next() {
        if style_pair.as_rule() == Rule::connection_style {
            style = parse_connection_style(style_pair)?;
        }
    }

    Ok(ConnectionDefinition {
        from,
        to: vec![to],
        style,
    })
}

fn parse_connections(pair: pest::iterators::Pair<Rule>) -> Result<Vec<ConnectionDefinition>> {
    let mut from = String::new();
    let mut to_list = Vec::new();
    let mut style = EdgeStyleDefinition {
        edge_type: None,
        label: None,
        label_position: None,
        routing: None,
        color: None,
        width: None,
        stroke_style: None,
    };

    // Based on the grammar, the pairs come in order: from string, to array, style block
    let mut inner_pairs = pair.into_inner();

    // First string literal is "from"
    if let Some(from_pair) = inner_pairs.next() {
        if from_pair.as_rule() == Rule::string_literal {
            from = from_pair.as_str().trim_matches('"').to_string();
        }
    }

    // Second is the connection_targets array
    if let Some(targets_pair) = inner_pairs.next() {
        if targets_pair.as_rule() == Rule::connection_targets {
            for target_pair in targets_pair.into_inner() {
                if target_pair.as_rule() == Rule::string_literal {
                    to_list.push(target_pair.as_str().trim_matches('"').to_string());
                }
            }
        }
    }

    // Third is the style block
    if let Some(style_pair) = inner_pairs.next() {
        if style_pair.as_rule() == Rule::connection_style {
            style = parse_connection_style(style_pair)?;
        }
    }

    // Create a connection for each target
    Ok(to_list
        .into_iter()
        .map(|to| ConnectionDefinition {
            from: from.clone(),
            to: vec![to],
            style: style.clone(),
        })
        .collect())
}

fn parse_connection_style(pair: pest::iterators::Pair<Rule>) -> Result<EdgeStyleDefinition> {
    let mut style = EdgeStyleDefinition {
        edge_type: None,
        label: None,
        label_position: None,
        routing: None,
        color: None,
        width: None,
        stroke_style: None,
    };

    for attr_pair in pair.into_inner() {
        if attr_pair.as_rule() == Rule::connection_style_attr {
            for inner in attr_pair.into_inner() {
                match inner.as_rule() {
                    Rule::edge_type => {
                        style.edge_type = Some(match inner.as_str() {
                            "arrow" => EdgeType::Arrow,
                            "line" => EdgeType::Line,
                            "dashed" => EdgeType::Dashed,
                            "dotted" => EdgeType::Dotted,
                            _ => EdgeType::Arrow,
                        });
                    }
                    Rule::string_literal => {
                        let value = parse_string_literal(inner.as_str())?;
                        // Determine which field based on context
                        if style.label.is_none() {
                            style.label = Some(value);
                        } else if style.color.is_none() {
                            style.color = Some(value);
                        }
                    }
                    Rule::number => {
                        let value: f64 =
                            inner.as_str().parse().map_err(|_| ParseError::Syntax {
                                line: inner.as_span().start_pos().line_col().0,
                                message: "Invalid number".to_string(),
                            })?;
                        // Determine which field based on context
                        if style.label_position.is_none() {
                            style.label_position = Some(value);
                        } else if style.width.is_none() {
                            style.width = Some(value);
                        }
                    }
                    Rule::routing_type => {
                        style.routing = Some(match inner.as_str() {
                            "straight" => RoutingType::Straight,
                            "orthogonal" => RoutingType::Orthogonal,
                            "curved" => RoutingType::Curved,
                            "auto" => RoutingType::Auto,
                            _ => RoutingType::Auto,
                        });
                    }
                    Rule::stroke_style => {
                        style.stroke_style = Some(match inner.as_str() {
                            "solid" => StrokeStyle::Solid,
                            "dashed" => StrokeStyle::Dashed,
                            "dotted" => StrokeStyle::Dotted,
                            _ => StrokeStyle::Solid,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(style)
}

fn parse_template_definition(pair: pest::iterators::Pair<Rule>) -> Result<TemplateDefinition> {
    let mut name = String::new();
    let mut layers = Vec::new();
    let mut connections = None;
    let mut layout = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                name = inner_pair.as_str().to_string();
            }
            Rule::template_body => {
                for body_pair in inner_pair.into_inner() {
                    match body_pair.as_rule() {
                        Rule::layers_def => {
                            for layer_pair in body_pair.into_inner() {
                                if layer_pair.as_rule() == Rule::layer_def {
                                    layers.push(parse_layer_definition(layer_pair)?);
                                }
                            }
                        }
                        Rule::connections_pattern => {
                            connections = Some(parse_connection_pattern(body_pair)?);
                        }
                        Rule::layout_def => {
                            layout = Some(parse_layout_definition(body_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(TemplateDefinition {
        name,
        layers,
        connections,
        layout,
    })
}

fn parse_layer_definition(pair: pest::iterators::Pair<Rule>) -> Result<LayerDefinition> {
    let mut name = String::new();
    let mut components = Vec::new();
    let mut layout = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::string_literal => {
                name = parse_string_literal(inner_pair.as_str())?;
            }
            Rule::layer_body => {
                for body_pair in inner_pair.into_inner() {
                    match body_pair.as_rule() {
                        Rule::components_list => {
                            for comp_pair in body_pair.into_inner() {
                                if comp_pair.as_rule() == Rule::string_literal {
                                    components.push(parse_string_literal(comp_pair.as_str())?);
                                }
                            }
                        }
                        Rule::layer_layout => {
                            layout = Some(parse_layer_layout(body_pair)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(LayerDefinition {
        name,
        components,
        layout,
    })
}

fn parse_layer_layout(pair: pest::iterators::Pair<Rule>) -> Result<LayerLayout> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_str() {
        "horizontal" => Ok(LayerLayout::Horizontal),
        "vertical" => Ok(LayerLayout::Vertical),
        layout_str if layout_str.starts_with("grid") => {
            // Parse grid(n) format
            let cols = layout_str
                .strip_prefix("grid(")
                .and_then(|s| s.strip_suffix(")"))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(2);
            Ok(LayerLayout::Grid { cols })
        }
        _ => Ok(LayerLayout::Horizontal),
    }
}

fn parse_connection_pattern(pair: pest::iterators::Pair<Rule>) -> Result<ConnectionPattern> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_str() {
        "each-to-next-layer" => Ok(ConnectionPattern::EachToNextLayer),
        "mesh" => Ok(ConnectionPattern::Mesh),
        pattern if pattern.starts_with("star(") => {
            let center = pattern
                .strip_prefix("star(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or("center")
                .to_string();
            Ok(ConnectionPattern::Star(center))
        }
        _ => Ok(ConnectionPattern::EachToNextLayer),
    }
}

fn parse_diagram_definition(pair: pest::iterators::Pair<Rule>) -> Result<DiagramDefinition> {
    let mut name = String::new();
    let mut diagram_type = DiagramType::Flow;
    let mut layout = None;
    let mut template = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::string_literal => {
                name = parse_string_literal(inner_pair.as_str())?;
            }
            Rule::diagram_body => {
                for body_pair in inner_pair.into_inner() {
                    match body_pair.as_rule() {
                        Rule::diagram_type => {
                            diagram_type = parse_diagram_type(body_pair)?;
                        }
                        Rule::layout_def => {
                            layout = Some(parse_layout_definition(body_pair)?);
                        }
                        Rule::template_ref => {
                            template =
                                Some(body_pair.into_inner().next().unwrap().as_str().to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(DiagramDefinition {
        name,
        diagram_type,
        layout,
        template,
    })
}

fn parse_diagram_type(pair: pest::iterators::Pair<Rule>) -> Result<DiagramType> {
    let type_name = pair.into_inner().next().unwrap().as_str();
    match type_name {
        "architecture" => Ok(DiagramType::Architecture),
        "flow" => Ok(DiagramType::Flow),
        "sequence" => Ok(DiagramType::Sequence),
        "network" => Ok(DiagramType::Network),
        "database" => Ok(DiagramType::Database),
        custom => Ok(DiagramType::Custom(custom.to_string())),
    }
}

fn parse_layout_definition(pair: pest::iterators::Pair<Rule>) -> Result<LayoutDefinition> {
    let mut layout_type = LayoutType::Layered;
    let mut direction = None;
    let mut spacing = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::layout_type => {
                layout_type = parse_layout_type(inner_pair)?;
            }
            Rule::layout_direction => {
                direction = Some(parse_layout_direction(inner_pair)?);
            }
            Rule::layout_spacing_def => {
                spacing = Some(parse_layout_spacing(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(LayoutDefinition {
        layout_type,
        direction,
        spacing,
    })
}

fn parse_layout_type(pair: pest::iterators::Pair<Rule>) -> Result<LayoutType> {
    let type_name = pair.into_inner().next().unwrap().as_str();
    match type_name {
        "layered" => Ok(LayoutType::Layered),
        "force" => Ok(LayoutType::Force),
        "grid" => Ok(LayoutType::Grid),
        "tree" => Ok(LayoutType::Tree),
        "manual" => Ok(LayoutType::Manual),
        _ => Ok(LayoutType::Layered),
    }
}

fn parse_layout_direction(pair: pest::iterators::Pair<Rule>) -> Result<LayoutDirection> {
    let direction_name = pair.into_inner().next().unwrap().as_str();
    match direction_name {
        "horizontal" => Ok(LayoutDirection::Horizontal),
        "vertical" => Ok(LayoutDirection::Vertical),
        "top-to-bottom" => Ok(LayoutDirection::TopToBottom),
        "bottom-to-top" => Ok(LayoutDirection::BottomToTop),
        "left-to-right" => Ok(LayoutDirection::LeftToRight),
        "right-to-left" => Ok(LayoutDirection::RightToLeft),
        _ => Ok(LayoutDirection::TopToBottom),
    }
}

fn parse_layout_spacing(pair: pest::iterators::Pair<Rule>) -> Result<LayoutSpacing> {
    let mut x = None;
    let mut y = None;
    let mut node_spacing = None;
    let mut layer_spacing = None;

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::spacing_params {
            for spacing_pair in inner_pair.into_inner() {
                match spacing_pair.as_rule() {
                    Rule::spacing_x => {
                        x = Some(
                            spacing_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .parse::<f64>()
                                .unwrap_or(50.0),
                        );
                    }
                    Rule::spacing_y => {
                        y = Some(
                            spacing_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .parse::<f64>()
                                .unwrap_or(50.0),
                        );
                    }
                    Rule::node_spacing => {
                        node_spacing = Some(
                            spacing_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .parse::<f64>()
                                .unwrap_or(30.0),
                        );
                    }
                    Rule::layer_spacing => {
                        layer_spacing = Some(
                            spacing_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .parse::<f64>()
                                .unwrap_or(100.0),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(LayoutSpacing {
        x,
        y,
        node_spacing,
        layer_spacing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_node() {
        let input = "web_server[Web Server]";
        let result = parse_edsl(input).unwrap();

        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, "web_server");
        assert_eq!(result.nodes[0].label, Some("Web Server".to_string()));
    }

    #[test]
    fn test_parse_simple_edge() {
        let input = r#"
        user
        api
        user -> api: "HTTP Request"
        "#;

        let result = parse_edsl(input).unwrap();

        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.edges[0].from, "user");
        assert_eq!(result.edges[0].to, "api");
        assert_eq!(result.edges[0].label, Some("HTTP Request".to_string()));
    }

    #[test]
    fn test_parse_with_config() {
        let input = r#"
        ---
        layout: dagre
        theme: dark
        ---

        web_server[Web Server]
        "#;

        let result = parse_edsl(input).unwrap();

        assert_eq!(result.config.layout, Some("dagre".to_string()));
        assert_eq!(result.config.theme, Some("dark".to_string()));
        assert_eq!(result.nodes.len(), 1);
    }
}
