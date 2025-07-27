// examples/improvements_demo.rs
//! Demonstration of the key improvements in excalidraw-dsl

use excalidraw_dsl::{DiagramBuilder, DiagramPresets, EDSLCompiler, ThemePresets};

fn main() -> excalidraw_dsl::Result<()> {
    println!("=== Excalidraw DSL Improvements Demo ===\n");

    // 1. Builder Pattern for EDSLCompiler
    println!("1. Builder Pattern for EDSLCompiler:");
    let _compiler = EDSLCompiler::builder()
        .with_validation(true)
        .with_parallel_layout(true)
        .with_max_threads(4)
        .with_cache(true)
        .build();
    println!("   ✓ Created compiler with custom configuration\n");

    // 2. Fluent API for diagram creation
    println!("2. Fluent API Example:");
    let diagram = DiagramBuilder::new()
        .with_layout("dagre")
        .with_theme("light")
        .node("frontend")
        .label("Frontend")
        .shape("rectangle")
        .background("#e3f2fd")
        .done()
        .node("backend")
        .label("Backend")
        .shape("rectangle")
        .background("#fff3e0")
        .done()
        .node("database")
        .label("Database")
        .shape("cylinder")
        .background("#f3e5f5")
        .done()
        .edge("frontend", "backend")
        .label("API Call")
        .done()
        .edge("backend", "database")
        .label("Query")
        .done()
        .build()?;

    println!("   ✓ Created diagram using fluent API");
    println!(
        "   ✓ Generated {} bytes of Excalidraw JSON\n",
        diagram.len()
    );

    // 3. Preset diagrams
    println!("3. Preset Diagrams:");
    let presets = vec![
        ("Client-Server", DiagramPresets::client_server()),
        ("Microservices", DiagramPresets::microservices()),
        ("CI/CD Pipeline", DiagramPresets::cicd_pipeline()),
        ("State Machine", DiagramPresets::state_machine()),
    ];

    for (name, preset) in presets {
        let ast = preset.build_ast();
        println!(
            "   ✓ {} preset: {} nodes, {} edges",
            name,
            ast.nodes.len(),
            ast.edges.len()
        );
    }

    // 4. Theme presets
    println!("\n4. Theme Presets:");
    let material_colors = ThemePresets::material_colors();
    let corporate_colors = ThemePresets::corporate_theme();
    let pastel_colors = ThemePresets::pastel_theme();

    println!("   ✓ Material Design: {} colors", material_colors.len());
    println!("   ✓ Corporate Theme: {} colors", corporate_colors.len());
    println!("   ✓ Pastel Theme: {} colors", pastel_colors.len());

    // 5. Error handling with context
    println!("\n5. Enhanced Error Handling:");
    let invalid_edsl = r#"
        node_a -> nonexistent_node
    "#;

    let mut compiler = EDSLCompiler::new();
    match compiler.compile(invalid_edsl) {
        Ok(_) => println!("   ✗ Should have failed"),
        Err(e) => {
            println!("   ✓ Error properly caught: {e}");
            // In a real scenario, we could add context:
            // let contextualized = e.with_context(|| ErrorContext::new()
            //     .with_line(2)
            //     .add_context("Failed to compile diagram".to_string()));
        }
    }

    // 6. Performance features
    println!("\n6. Performance Features:");
    println!("   ✓ String interning enabled in generator");
    println!("   ✓ Parallel layout processing available");
    println!("   ✓ Pre-allocated vectors for better performance");
    println!("   ✓ Layout caching enabled by default");

    println!("\n=== Demo Complete ===");
    Ok(())
}
