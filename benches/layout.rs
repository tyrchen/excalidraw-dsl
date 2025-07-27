use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use excalidraw_dsl::{igr::IntermediateGraph, layout::LayoutManager, parser::parse_edsl};

fn create_simple_igr() -> IntermediateGraph {
    let edsl = r#"
---
layout: dagre
---

a[Node A]
b[Node B]
c[Node C]
d[Node D]

a -> b -> c -> d
a -> c
b -> d
"#;

    let ast = parse_edsl(edsl).expect("Failed to parse simple EDSL");
    IntermediateGraph::from_ast(ast).expect("Failed to create IGR")
}

fn create_simple_force_igr() -> IntermediateGraph {
    let edsl = r#"
---
layout: force
---

a[Node A]
b[Node B]
c[Node C]
d[Node D]

a -> b -> c -> d
a -> c
b -> d
"#;

    let ast = parse_edsl(edsl).expect("Failed to parse simple EDSL");
    IntermediateGraph::from_ast(ast).expect("Failed to create IGR")
}

fn create_medium_igr() -> IntermediateGraph {
    let edsl = r#"
---
layout: dagre
---

start[Start]
process1[Process 1]
process2[Process 2]
process3[Process 3]
decision[Decision Point]
result1[Result 1]
result2[Result 2]
end[End]

start -> process1
process1 -> process2
process2 -> process3
process3 -> decision
decision -> result1
decision -> result2
result1 -> end
result2 -> end

# Add some cross connections for complexity
start -> decision
process1 -> result1
"#;

    let ast = parse_edsl(edsl).expect("Failed to parse medium EDSL");
    IntermediateGraph::from_ast(ast).expect("Failed to create IGR")
}

fn create_large_igr() -> IntermediateGraph {
    let edsl = r#"
---
layout: dagre
---

container "Main Flow" {
    input[Input]
    validate[Validate]
    process[Process]
    output[Output]

    input -> validate -> process -> output
}

container "Error Handling" {
    error_catch[Error Catch]
    error_log[Error Log]
    error_notify[Error Notify]
    fallback[Fallback]

    error_catch -> error_log
    error_catch -> error_notify
    error_catch -> fallback
}

container "Monitoring" {
    metrics[Metrics]
    dashboard[Dashboard]
    alerts[Alerts]
    reports[Reports]

    metrics -> dashboard
    metrics -> alerts
    metrics -> reports
}

# Cross-container connections
validate -> error_catch
process -> metrics
fallback -> input
alerts -> error_notify
"#;

    let ast = parse_edsl(edsl).expect("Failed to parse large EDSL");
    IntermediateGraph::from_ast(ast).expect("Failed to create IGR")
}

fn create_scaling_igr(node_count: usize, layout: &str) -> IntermediateGraph {
    let mut edsl = format!("---\nlayout: {layout}\n---\n\n");

    // Generate nodes
    for i in 0..node_count {
        edsl.push_str(&format!("node{i}[Node {i}]\n"));
    }

    edsl.push('\n');

    // Generate edges - create a more complex graph structure
    // Chain connections
    for i in 0..(node_count - 1) {
        edsl.push_str(&format!("node{} -> node{}\n", i, i + 1));
    }

    // Cross connections for complexity
    for i in (0..node_count).step_by(3) {
        if i + 2 < node_count {
            edsl.push_str(&format!("node{} -> node{}\n", i, i + 2));
        }
        if i + 4 < node_count {
            edsl.push_str(&format!("node{} -> node{}\n", i, i + 4));
        }
    }

    // Reverse connections for cycles (force layout)
    if node_count > 10 {
        for i in (5..node_count).step_by(7) {
            if i >= 3 {
                edsl.push_str(&format!("node{} -> node{}\n", i, i - 3));
            }
        }
    }

    let ast = parse_edsl(&edsl).expect("Failed to parse scaling EDSL");
    IntermediateGraph::from_ast(ast).expect("Failed to create IGR")
}

fn bench_layout_dagre_simple(c: &mut Criterion) {
    c.bench_function("layout_dagre_simple", |b| {
        b.iter(|| {
            let mut igr = create_simple_igr();
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });
}

fn bench_layout_dagre_medium(c: &mut Criterion) {
    c.bench_function("layout_dagre_medium", |b| {
        b.iter(|| {
            let mut igr = create_medium_igr();
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });
}

fn bench_layout_dagre_large(c: &mut Criterion) {
    c.bench_function("layout_dagre_large", |b| {
        b.iter(|| {
            let mut igr = create_large_igr();
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });
}

fn bench_layout_force_simple(c: &mut Criterion) {
    c.bench_function("layout_force_simple", |b| {
        b.iter(|| {
            let mut igr = create_simple_force_igr();
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });
}

fn bench_layout_force_medium(c: &mut Criterion) {
    c.bench_function("layout_force_medium", |b| {
        b.iter(|| {
            let mut igr = create_scaling_igr(10, "force");
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });
}

fn bench_layout_force_large(c: &mut Criterion) {
    c.bench_function("layout_force_large", |b| {
        b.iter(|| {
            let mut igr = create_scaling_igr(20, "force");
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });
}

fn bench_layout_dagre_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_dagre_scaling");

    for node_count in [5, 10, 20, 40, 80].iter() {
        group.bench_with_input(
            BenchmarkId::new("nodes", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    let mut igr = create_scaling_igr(node_count, "dagre");
                    let manager = LayoutManager::new();
                    black_box(manager.layout(&mut igr))
                })
            },
        );
    }
    group.finish();
}

fn bench_layout_force_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_force_scaling");

    // Use smaller node counts for force layout as it's more computationally expensive
    for node_count in [5, 10, 15, 25, 40].iter() {
        group.bench_with_input(
            BenchmarkId::new("nodes", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    let mut igr = create_scaling_igr(node_count, "force");
                    let manager = LayoutManager::new();
                    black_box(manager.layout(&mut igr))
                })
            },
        );
    }
    group.finish();
}

fn bench_layout_algorithm_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_algorithm_comparison");

    group.bench_function("dagre", |b| {
        b.iter(|| {
            let mut igr = create_medium_igr();
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });

    group.bench_function("force", |b| {
        b.iter(|| {
            let mut igr = create_scaling_igr(15, "force"); // Medium size with force layout
            let manager = LayoutManager::new();
            black_box(manager.layout(&mut igr))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_layout_dagre_simple,
    bench_layout_dagre_medium,
    bench_layout_dagre_large,
    bench_layout_force_simple,
    bench_layout_force_medium,
    bench_layout_force_large,
    bench_layout_dagre_scaling,
    bench_layout_force_scaling,
    bench_layout_algorithm_comparison
);
criterion_main!(benches);
