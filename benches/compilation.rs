use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use excalidraw_dsl::EDSLCompiler;

const SIMPLE_EDSL: &str = r#"
a[A]
b[B]
c[C]
a -> b -> c
"#;

const MEDIUM_EDSL: &str = r#"
// Medium complexity diagram
start[Start Process]
validate[Validate Input]
process[Process Data]
save[Save Results]
notify[Send Notification]
error[Error Handler]
end[End]

start -> validate
validate -> process: "valid"
validate -> error: "invalid"
process -> save
save -> notify
notify -> end
error -> end
"#;

const LARGE_EDSL: &str = r#"
// Large diagram with containers and complex flows
container "Data Processing Pipeline" {
    input[Input Source]
    parser[Data Parser]
    validator[Data Validator]
    transformer[Data Transformer]
    output[Output Sink]

    input -> parser -> validator -> transformer -> output
}

container "Error Handling" {
    error_detector[Error Detector]
    logger[Error Logger]
    notifier[Error Notifier]
    fallback[Fallback Handler]

    error_detector -> logger
    error_detector -> notifier
    error_detector -> fallback
}

container "Monitoring" {
    metrics[Metrics Collector]
    dashboard[Dashboard]
    alerts[Alert Manager]

    metrics -> dashboard
    metrics -> alerts
}

// Cross-container connections
validator -> error_detector: "validation error"
transformer -> metrics: "processing metrics"
fallback -> input: "retry"
"#;

const COMPLEX_EDSL: &str = r#"
---
layout: dagre
direction: TB
---

# Complex enterprise workflow diagram
container "Frontend Layer" {
    web[Web Interface]
    mobile[Mobile App]
    api_gateway[API Gateway]

    web -> api_gateway
    mobile -> api_gateway
}

container "Business Logic" {
    auth[Authentication]
    user_service[User Service]
    order_service[Order Service]
    payment_service[Payment Service]
    inventory[Inventory Service]
    notification[Notification Service]

    auth -> user_service
    user_service -> order_service
    order_service -> payment_service
    order_service -> inventory
    payment_service -> notification
}

container "Data Layer" {
    user_db[(User Database)]
    order_db[(Order Database)]
    payment_db[(Payment Database)]
    cache[(Redis Cache)]

    user_service -> user_db
    user_service -> cache
    order_service -> order_db
    payment_service -> payment_db
}

container "External Services" {
    payment_gateway[Payment Gateway]
    email_service[Email Service]
    sms_service[SMS Service]

    payment_service -> payment_gateway
    notification -> email_service
    notification -> sms_service
}

# Cross-container flows
api_gateway -> auth
api_gateway -> user_service
api_gateway -> order_service
"#;

fn bench_compilation_simple(c: &mut Criterion) {
    c.bench_function("compile_simple", |b| {
        b.iter(|| {
            let mut compiler = EDSLCompiler::new();
            black_box(compiler.compile(black_box(SIMPLE_EDSL)))
        })
    });
}

fn bench_compilation_medium(c: &mut Criterion) {
    c.bench_function("compile_medium", |b| {
        b.iter(|| {
            let mut compiler = EDSLCompiler::new();
            black_box(compiler.compile(black_box(MEDIUM_EDSL)))
        })
    });
}

fn bench_compilation_large(c: &mut Criterion) {
    c.bench_function("compile_large", |b| {
        b.iter(|| {
            let mut compiler = EDSLCompiler::new();
            black_box(compiler.compile(black_box(LARGE_EDSL)))
        })
    });
}

fn bench_compilation_complex(c: &mut Criterion) {
    c.bench_function("compile_complex", |b| {
        b.iter(|| {
            let mut compiler = EDSLCompiler::new();
            black_box(compiler.compile(black_box(COMPLEX_EDSL)))
        })
    });
}

fn bench_compilation_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_scaling");

    for node_count in [5, 10, 20, 50, 100].iter() {
        // Generate a diagram with specified number of nodes
        let edsl = generate_scaling_edsl(*node_count);
        group.bench_with_input(BenchmarkId::new("nodes", node_count), &edsl, |b, edsl| {
            b.iter(|| {
                let mut compiler = EDSLCompiler::new();
                black_box(compiler.compile(black_box(edsl)))
            })
        });
    }
    group.finish();
}

fn generate_scaling_edsl(node_count: usize) -> String {
    let mut edsl = String::new();

    // Generate nodes
    for i in 0..node_count {
        edsl.push_str(&format!("node{i}[Node {i}]\n"));
    }

    edsl.push('\n');

    // Generate edges in a chain pattern
    for i in 0..(node_count - 1) {
        edsl.push_str(&format!("node{} -> node{}\n", i, i + 1));
    }

    // Add some cross connections for complexity
    if node_count > 4 {
        for i in (0..node_count).step_by(3) {
            if i + 2 < node_count {
                edsl.push_str(&format!("node{} -> node{}\n", i, i + 2));
            }
        }
    }

    edsl
}

criterion_group!(
    benches,
    bench_compilation_simple,
    bench_compilation_medium,
    bench_compilation_large,
    bench_compilation_complex,
    bench_compilation_scaling
);
criterion_main!(benches);
