use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use reactive_tui::core::grapheme_cell::GraphemeSurface;
use reactive_tui::core::span_diff::SpanDiffWriter;
use reactive_tui::core::surface::{Attr, DiffWriter, Rgba, Surface};

fn create_test_surface_grapheme(width: usize, height: usize, text: &str) -> GraphemeSurface {
    let mut surface = GraphemeSurface::new(width, height);
    let fg = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let bg = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    for y in 0..height {
        surface.write_str(0, y, text, fg, bg, Attr::empty());
    }
    surface
}

fn create_test_surface_old(width: usize, height: usize, text: &str) -> Surface {
    let mut surface = Surface::new(width, height);
    let fg = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let bg = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    for y in 0..height {
        surface.write_str(0, y, text, fg, bg, Attr::empty());
    }
    surface
}

fn bench_span_diff_ascii(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_diff_ascii");

    for size in [10, 40, 80].iter() {
        let old = create_test_surface_grapheme(*size, 24, "Hello World");
        let new = create_test_surface_grapheme(*size, 24, "Hello Rust!");

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut writer = SpanDiffWriter::new();
            b.iter(|| {
                writer.diff(black_box(&old), black_box(&new));
                black_box(writer.output());
                writer.clear();
            });
        });
    }
    group.finish();
}

fn bench_span_diff_cjk(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_diff_cjk");

    for size in [20, 40, 80].iter() {
        let old = create_test_surface_grapheme(*size, 24, "你好世界");
        let new = create_test_surface_grapheme(*size, 24, "世界你好");

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut writer = SpanDiffWriter::new();
            b.iter(|| {
                writer.diff(black_box(&old), black_box(&new));
                black_box(writer.output());
                writer.clear();
            });
        });
    }
    group.finish();
}

fn bench_span_diff_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_diff_mixed");

    let old_text = "Hello 世界! 🎉 Rust";
    let new_text = "Hi 世界! 😀 Rust!";

    for size in [30, 60, 120].iter() {
        let old = create_test_surface_grapheme(*size, 24, old_text);
        let new = create_test_surface_grapheme(*size, 24, new_text);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut writer = SpanDiffWriter::new();
            b.iter(|| {
                writer.diff(black_box(&old), black_box(&new));
                black_box(writer.output());
                writer.clear();
            });
        });
    }
    group.finish();
}

fn bench_old_vs_new_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_comparison");

    // Old cell-by-cell approach
    let old_surface1 = create_test_surface_old(80, 24, "Hello World");
    let old_surface2 = create_test_surface_old(80, 24, "Hello Rust!");

    group.bench_function("old_cell_by_cell", |b| {
        let mut writer = DiffWriter::new();
        b.iter(|| {
            writer.diff(black_box(&old_surface1), black_box(&old_surface2), false);
            black_box(writer.output());
        });
    });

    // New row-span approach
    let new_surface1 = create_test_surface_grapheme(80, 24, "Hello World");
    let new_surface2 = create_test_surface_grapheme(80, 24, "Hello Rust!");

    group.bench_function("new_row_span", |b| {
        let mut writer = SpanDiffWriter::new();
        b.iter(|| {
            writer.diff(black_box(&new_surface1), black_box(&new_surface2));
            black_box(writer.output());
            writer.clear();
        });
    });

    group.finish();
}

fn bench_style_changes(c: &mut Criterion) {
    let mut group = c.benchmark_group("style_changes");

    // Create surfaces with alternating styles to test coalescing
    let mut old = GraphemeSurface::new(80, 24);
    let mut new = GraphemeSurface::new(80, 24);

    let fg1 = Rgba {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let fg2 = Rgba {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    let bg = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    for y in 0..24 {
        // Old: alternating colors per word
        old.write_str(0, y, "Hello", fg1, bg, Attr::empty());
        old.write_str(6, y, "World", fg2, bg, Attr::empty());

        // New: same color for entire line (tests coalescing)
        new.write_str(0, y, "Hello World", fg1, bg, Attr::empty());
    }

    group.bench_function("many_style_changes", |b| {
        let mut writer = SpanDiffWriter::new();
        b.iter(|| {
            writer.diff(black_box(&old), black_box(&new));
            black_box(writer.output());
            writer.clear();
        });
    });

    group.finish();
}

fn bench_output_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_size");

    let old = create_test_surface_grapheme(80, 24, "A");
    let new = create_test_surface_grapheme(80, 24, "B");

    group.bench_function("measure_bytes", |b| {
        let mut writer = SpanDiffWriter::new();
        b.iter(|| {
            let stats = writer.diff_with_stats(black_box(&old), black_box(&new));
            black_box(stats.bytes_written);
            writer.clear();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_span_diff_ascii,
    bench_span_diff_cjk,
    bench_span_diff_mixed,
    bench_old_vs_new_diff,
    bench_style_changes,
    bench_output_size
);
criterion_main!(benches);
