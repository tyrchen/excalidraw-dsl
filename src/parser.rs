// src/parser.rs
use crate::ast::*;
use crate::error::{ParseError, Result};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "edsl.pest"]
pub struct EDSLParser;

pub fn parse_edsl(input: &str) -> Result<ParsedDocument> {
    let pairs = EDSLParser::parse(Rule::file, input).map_err(ParseError::PestError)?;

    build_document(pairs)
}

fn build_document(pairs: pest::iterators::Pairs<Rule>) -> Result<ParsedDocument> {
    let mut config = GlobalConfig::default();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut containers = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::file => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::config => {
                            config = parse_config(inner_pair)?;
                        }
                        Rule::statement => {
                            let statement = parse_statement(inner_pair)?;
                            match statement {
                                Statement::Node(node) => nodes.push(node),
                                Statement::Edge(edge) => edges.push(edge),
                                Statement::Container(container) => containers.push(container),
                            }
                        }
                        Rule::EOI => break,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ParsedDocument {
        config,
        nodes,
        edges,
        containers,
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
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::node_def => Ok(Statement::Node(parse_node_definition(inner)?)),
        Rule::edge_def => Ok(Statement::Edge(parse_edge_definition(inner)?)),
        Rule::container_def => Ok(Statement::Container(parse_container_definition(inner)?)),
        _ => unreachable!(),
    }
}

fn parse_node_definition(pair: pest::iterators::Pair<Rule>) -> Result<NodeDefinition> {
    let mut id = String::new();
    let mut label = None;
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
            Rule::style_block => {
                attributes = parse_style_block(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(NodeDefinition {
        id,
        label,
        attributes,
    })
}

fn parse_edge_definition(pair: pest::iterators::Pair<Rule>) -> Result<EdgeDefinition> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::single_edge => parse_single_edge(inner),
        Rule::edge_chain => parse_edge_chain(inner),
        _ => unreachable!(),
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
                arrow_type =
                    ArrowType::from_str(inner_pair.as_str()).unwrap_or(ArrowType::SingleArrow);
            }
            Rule::edge_label => {
                for label_part in inner_pair.into_inner() {
                    match label_part.as_rule() {
                        Rule::edge_label_content => {
                            let content = label_part.as_str();
                            if content.starts_with('"') && content.ends_with('"') {
                                label = Some(parse_string_literal(content)?);
                            } else {
                                label = Some(content.trim().to_string());
                            }
                        }
                        _ => {}
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
    })
}

fn parse_edge_chain(pair: pest::iterators::Pair<Rule>) -> Result<EdgeDefinition> {
    // For now, just parse as a single edge (first two nodes)
    // TODO: Expand chains into multiple edges
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
                arrow_type =
                    ArrowType::from_str(inner_pair.as_str()).unwrap_or(ArrowType::SingleArrow);
            }
            Rule::edge_label => {
                for label_part in inner_pair.into_inner() {
                    match label_part.as_rule() {
                        Rule::edge_label_content => {
                            let content = label_part.as_str();
                            if content.starts_with('"') && content.ends_with('"') {
                                label = Some(parse_string_literal(content)?);
                            } else {
                                label = Some(content.trim().to_string());
                            }
                        }
                        _ => {}
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
        Ok(EdgeDefinition {
            from: ids[0].clone(),
            to: ids[1].clone(),
            label,
            arrow_type,
            attributes,
        })
    } else {
        Err(ParseError::Syntax {
            line: 0,
            message: "Edge chain requires at least two nodes".to_string(),
        }
        .into())
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
                    .unwrap();
                attributes = parse_style_block(style_block)?;
            }
            Rule::statement => {
                let statement = parse_statement(inner_pair)?;

                // Collect child node IDs
                match &statement {
                    Statement::Node(node) => {
                        children.push(node.id.clone());
                    }
                    Statement::Edge(edge) => {
                        // Ensure both nodes are tracked as children
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
                message: format!("Invalid number: {}", num_str),
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
