use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use excalidraw_dsl::parser::parse_edsl;

const SIMPLE_SYNTAX: &str = r#"
a[Simple Node]
b[Another Node]
a -> b
"#;

const COMPLEX_SYNTAX: &str = r#"
---
layout: dagre
direction: TB
node_spacing: 100
edge_spacing: 50
---

# Complex parsing test with all syntax features
container "Main Container" {
    node1[Node with Label]
    node2[Another Node]: "with description"

    node1 -> node2: "labeled edge"
    node1 -- node2: "line edge"
    node1 <-> node2: "bidirectional"
}

group "Semantic Group" {
    grouped_a[Grouped A]
    grouped_b[Grouped B]
    grouped_c[Grouped C]

    grouped_a -> grouped_b -> grouped_c
}

# External connections
node1 -> grouped_a
grouped_c -> node2

# Edge chains
chain1[Chain Start] -> chain2[Chain Middle] -> chain3[Chain End]

# Comments and whitespace handling
/* Multi-line comment
   with complex content */

standalone[Standalone Node]

# Various arrow types
arrow_test[Arrow Test]
line_test[Line Test]
bidir_test[Bidirectional Test]

arrow_test -> line_test
line_test -- bidir_test
arrow_test <-> bidir_test
"#;

fn bench_parse_simple(c: &mut Criterion) {
    c.bench_function("parse_simple", |b| {
        b.iter(|| black_box(parse_edsl(black_box(SIMPLE_SYNTAX))))
    });
}

fn bench_parse_complex(c: &mut Criterion) {
    c.bench_function("parse_complex", |b| {
        b.iter(|| black_box(parse_edsl(black_box(COMPLEX_SYNTAX))))
    });
}

fn bench_parse_frontmatter(c: &mut Criterion) {
    let yaml_frontmatter = r#"
---
layout: dagre
direction: LR
node_spacing: 80
edge_spacing: 40
padding: 20
---

simple[Simple Node]
"#;

    c.bench_function("parse_frontmatter", |b| {
        b.iter(|| black_box(parse_edsl(black_box(yaml_frontmatter))))
    });
}

fn bench_parse_containers(c: &mut Criterion) {
    let container_syntax = r#"
container "Container 1" {
    a[Node A]
    b[Node B]
    a -> b
}

container "Container 2" {
    c[Node C]
    d[Node D]
    c -> d
}

a -> c
"#;

    c.bench_function("parse_containers", |b| {
        b.iter(|| black_box(parse_edsl(black_box(container_syntax))))
    });
}

fn bench_parse_edge_chains(c: &mut Criterion) {
    let edge_chain_syntax = r#"
n1[Node 1]
n2[Node 2]
n3[Node 3]
n4[Node 4]
n5[Node 5]
n6[Node 6]
n7[Node 7]
n8[Node 8]

n1 -> n2 -> n3 -> n4 -> n5 -> n6 -> n7 -> n8
n1 -- n3 -- n5 -- n7
n2 <-> n4 <-> n6 <-> n8
"#;

    c.bench_function("parse_edge_chains", |b| {
        b.iter(|| black_box(parse_edsl(black_box(edge_chain_syntax))))
    });
}

fn bench_parse_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_scaling");

    for node_count in [10, 25, 50, 100, 200].iter() {
        let edsl = generate_parsing_edsl(*node_count);
        group.bench_with_input(BenchmarkId::new("nodes", node_count), &edsl, |b, edsl| {
            b.iter(|| black_box(parse_edsl(black_box(edsl))))
        });
    }
    group.finish();
}

fn generate_parsing_edsl(node_count: usize) -> String {
    let mut edsl = String::new();

    // Add frontmatter
    edsl.push_str("---\nlayout: dagre\ndirection: TB\n---\n\n");

    // Generate containers with nodes
    let containers = (node_count / 10).max(1);
    let nodes_per_container = node_count / containers;

    for c in 0..containers {
        edsl.push_str(&format!("container \"Container {c}\" {{\n"));

        let start_node = c * nodes_per_container;
        let end_node = ((c + 1) * nodes_per_container).min(node_count);

        for i in start_node..end_node {
            edsl.push_str(&format!("    node{i}[Node {i} in Container {c}]\n"));
        }

        edsl.push('\n');

        // Add edges within container
        for i in start_node..(end_node - 1) {
            edsl.push_str(&format!("    node{} -> node{}\n", i, i + 1));
        }

        edsl.push_str("}\n\n");
    }

    // Add cross-container connections
    for c in 0..(containers - 1) {
        let from_node = c * nodes_per_container;
        let to_node = (c + 1) * nodes_per_container;
        edsl.push_str(&format!("node{from_node} -> node{to_node}\n"));
    }

    edsl
}

fn bench_parse_comments(c: &mut Criterion) {
    let comment_heavy = r#"
# This is a single line comment
// This is also a single line comment

/* This is a
   multi-line comment
   with lots of text */

a[Node A] # Inline comment
b[Node B] // Another inline comment

# Comment before edge
a -> b

/* Another multi-line comment
   before the end */
"#;

    c.bench_function("parse_comments", |b| {
        b.iter(|| black_box(parse_edsl(black_box(comment_heavy))))
    });
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_complex,
    bench_parse_frontmatter,
    bench_parse_containers,
    bench_parse_edge_chains,
    bench_parse_scaling,
    bench_parse_comments
);
criterion_main!(benches);
