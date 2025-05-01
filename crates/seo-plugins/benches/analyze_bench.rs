use criterion::async_executor::FuturesExecutor;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use seo_plugins::utils::{page::Page, registry::PluginRegistry};

async fn analyze_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze");

    // Setup
    let registry = PluginRegistry::default_with_config();
    // let mut config = RuleConfig::new();
    // for rule in registry.get_available_rules().await {
    //     config.enable_rule(rule.id);
    // }

    // registry.set_config(config);

    let page = Page::from_html(
        r#"
        <html>
            <head>
                <title>Test Page</title>
                <meta name="description" content="Test description">
            </head>
            <body>
                <img src="/test.jpg" alt="Test Image">
                <img src="/test2.jpg" srcset="test2.jpg 1x, test2-2x.jpg 2x">
            </body>
        </html>
        "#
        .to_string(),
    );

    // Benchmark sync analyze
    group.bench_function("sync_analyze", |b| {
        b.iter(|| {
            black_box(registry.analyze(&page));
        })
    });

    // Benchmark async analyze
    group.bench_function("async_analyze", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            black_box(registry.analyze_async(&page).await);
        })
    });

    group.finish();
}

criterion_group!(benches, analyze_benchmark);
criterion_main!(benches);
