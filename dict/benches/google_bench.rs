use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use hjdict::google::{Output, merge_output};
use std::hint::black_box;

fn merge_output_original(outputs: Vec<Output>) -> (String, String) {
    let mut merged_source = String::new();
    let mut merged_translation = String::new();

    for output in outputs {
        merged_source.push_str(&output.source);
        merged_translation.push_str(&output.translation);
    }

    (merged_source, merged_translation)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut outputs = Vec::new();
    for i in 0..100 {
        outputs.push(Output {
            source: format!(
                "Source sentence number {} which is reasonably long enough. ",
                i
            ),
            translation: format!(
                "Translation of sentence {} is also quite a bit long to show reallocations. ",
                i
            ),
        });
    }

    c.bench_function("merge_output_original", |b| {
        b.iter_batched(
            || outputs.clone(),
            |outputs_cloned| merge_output_original(black_box(outputs_cloned)),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("merge_output_optimized", |b| {
        b.iter_batched(
            || outputs.clone(),
            |outputs_cloned| merge_output(black_box(outputs_cloned)),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
