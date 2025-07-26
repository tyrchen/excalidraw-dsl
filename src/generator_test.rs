#[cfg(test)]
mod tests {
    use crate::generator::ExcalidrawGenerator;
    use crate::ast::*;
    use crate::igr::{ExcalidrawAttributes, IntermediateGraph, NodeData};
    use std::collections::HashMap;

    #[test]
    fn test_text_dimensions_calculation() {
        // Test character width calculation
        let (width, height) = ExcalidrawGenerator::calculate_text_dimensions("Hello", 20.0, 3);
        assert!(width > 0);
        assert_eq!(height, 24); // 20 * 1.2

        // Test narrow characters
        let (narrow_width, _) = ExcalidrawGenerator::calculate_text_dimensions("iii", 20.0, 3);
        let (normal_width, _) = ExcalidrawGenerator::calculate_text_dimensions("aaa", 20.0, 3);
        assert!(narrow_width < normal_width);

        // Test wide characters
        let (wide_width, _) = ExcalidrawGenerator::calculate_text_dimensions("WWW", 20.0, 3);
        let (normal_width2, _) = ExcalidrawGenerator::calculate_text_dimensions("AAA", 20.0, 3);
        assert!(wide_width > normal_width2);
    }

    #[test]
    fn test_font_family_conversion() {
        assert_eq!(ExcalidrawGenerator::convert_font_family(&Some("Virgil".to_string())), 1);
        assert_eq!(ExcalidrawGenerator::convert_font_family(&Some("Helvetica".to_string())), 2);
        assert_eq!(ExcalidrawGenerator::convert_font_family(&Some("Cascadia".to_string())), 3);
        assert_eq!(ExcalidrawGenerator::convert_font_family(&Some("Code".to_string())), 3);
        assert_eq!(ExcalidrawGenerator::convert_font_family(&None), 3); // Default
        assert_eq!(ExcalidrawGenerator::convert_font_family(&Some("Unknown".to_string())), 3); // Fallback
    }

    #[test]
    fn test_generate_text_element() {
        let result = ExcalidrawGenerator::generate_text_element(
            "Test Text",
            100.0,
            200.0,
            "container123",
            20.0,
            &Some("Code".to_string()),
        );

        assert!(result.is_ok());
        let element = result.unwrap();
        
        assert_eq!(element.r#type, "text");
        assert_eq!(element.text, Some("Test Text".to_string()));
        assert_eq!(element.font_family, 3); // Code font
        assert_eq!(element.font_size, 20);
        assert_eq!(element.container_id, Some("container123".to_string()));
        assert_eq!(element.text_align, Some("center".to_string()));
        assert_eq!(element.vertical_align, Some("middle".to_string()));
    }

    #[test]
    fn test_generate_node_element() {
        let node_data = NodeData {
            id: "test_node".to_string(),
            label: "Test Node".to_string(),
            attributes: ExcalidrawAttributes {
                shape: Some("rectangle".to_string()),
                stroke_color: Some("#ff0000".to_string()),
                background_color: Some("#00ff00".to_string()),
                roughness: Some(0),
                font_size: Some(24.0),
                ..Default::default()
            },
            x: 100.0,
            y: 200.0,
            width: 150.0,
            height: 80.0,
        };

        let result = ExcalidrawGenerator::generate_node(&node_data, "test_id");
        assert!(result.is_ok());
        
        let element = result.unwrap();
        assert_eq!(element.r#type, "rectangle");
        assert_eq!(element.id, "test_id");
        assert_eq!(element.x, 25); // 100 - 75 (half width)
        assert_eq!(element.y, 160); // 200 - 40 (half height)
        assert_eq!(element.width, 150);
        assert_eq!(element.height, 80);
        assert_eq!(element.stroke_color, "#ff0000");
        assert_eq!(element.background_color, "#00ff00");
        assert_eq!(element.roughness, 0);
        assert_eq!(element.font_size, 24);
        assert_eq!(element.font_family, 3); // Default Cascadia
    }

    #[test]
    fn test_generate_file_structure() {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            nodes: vec![NodeDefinition {
                id: "test".to_string(),
                label: Some("Test Node".to_string()),
                attributes: HashMap::new(),
            }],
            edges: vec![],
            containers: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let result = ExcalidrawGenerator::generate_file(&igr);
        
        assert!(result.is_ok());
        let file = result.unwrap();
        
        assert_eq!(file.r#type, "excalidraw");
        assert_eq!(file.version, 2);
        assert_eq!(file.source, "https://excalidraw-dsl.com");
        assert!(!file.elements.is_empty());
        assert_eq!(file.app_state.view_background_color, "#ffffff");
    }

    #[test]
    fn test_shape_type_conversion() {
        let mut attrs = ExcalidrawAttributes::default();
        
        // Test rectangle (default)
        attrs.shape = None;
        let node_data = create_test_node(&attrs);
        let element = ExcalidrawGenerator::generate_node(&node_data, "test").unwrap();
        assert_eq!(element.r#type, "rectangle");

        // Test ellipse
        attrs.shape = Some("ellipse".to_string());
        let node_data = create_test_node(&attrs);
        let element = ExcalidrawGenerator::generate_node(&node_data, "test").unwrap();
        assert_eq!(element.r#type, "ellipse");

        // Test diamond
        attrs.shape = Some("diamond".to_string());
        let node_data = create_test_node(&attrs);
        let element = ExcalidrawGenerator::generate_node(&node_data, "test").unwrap();
        assert_eq!(element.r#type, "diamond");

        // Test invalid shape (should error)
        attrs.shape = Some("invalid_shape".to_string());
        let node_data = create_test_node(&attrs);
        let result = ExcalidrawGenerator::generate_node(&node_data, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_coordinate_validation() {
        let mut node_data = create_test_node(&ExcalidrawAttributes::default());
        
        // Test invalid coordinates
        node_data.x = f64::NAN;
        let result = ExcalidrawGenerator::generate_node(&node_data, "test");
        assert!(result.is_err());

        node_data.x = 100.0;
        node_data.y = f64::INFINITY;
        let result = ExcalidrawGenerator::generate_node(&node_data, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_fill_style_conversion() {
        assert_eq!(ExcalidrawGenerator::convert_fill_style(&Some(FillStyle::Solid)), "solid");
        assert_eq!(ExcalidrawGenerator::convert_fill_style(&Some(FillStyle::Hachure)), "hachure");
        assert_eq!(ExcalidrawGenerator::convert_fill_style(&Some(FillStyle::CrossHatch)), "cross-hatch");
        assert_eq!(ExcalidrawGenerator::convert_fill_style(&None), "solid");
    }

    #[test]
    fn test_stroke_style_conversion() {
        assert_eq!(ExcalidrawGenerator::convert_stroke_style(&Some(StrokeStyle::Solid)), "solid");
        assert_eq!(ExcalidrawGenerator::convert_stroke_style(&Some(StrokeStyle::Dashed)), "dashed");
        assert_eq!(ExcalidrawGenerator::convert_stroke_style(&Some(StrokeStyle::Dotted)), "dotted");
        assert_eq!(ExcalidrawGenerator::convert_stroke_style(&None), "solid");
    }

    fn create_test_node(attrs: &ExcalidrawAttributes) -> NodeData {
        NodeData {
            id: "test".to_string(),
            label: "Test".to_string(),
            attributes: attrs.clone(),
            x: 100.0,
            y: 100.0,
            width: 80.0,
            height: 60.0,
        }
    }
}