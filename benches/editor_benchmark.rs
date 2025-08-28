use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use reactive_tui::editor::gap_buffer::GapBuffer;

/// Alternative: Vec<String> based buffer for comparison
#[derive(Clone)]
struct VecBuffer {
    lines: Vec<String>,
}

impl VecBuffer {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    fn from_str(s: &str) -> Self {
        Self {
            lines: s.lines().map(String::from).collect(),
        }
    }

    fn insert_at(&mut self, line: usize, col: usize, text: &str) {
        if line >= self.lines.len() {
            self.lines.resize(line + 1, String::new());
        }

        let line_text = &mut self.lines[line];
        if col <= line_text.len() {
            line_text.insert_str(col, text);
        } else {
            line_text.push_str(text);
        }
    }

    fn delete_at(&mut self, line: usize, col: usize) -> Option<char> {
        if line < self.lines.len() {
            let line_text = &mut self.lines[line];
            if col < line_text.len() {
                let ch = line_text.chars().nth(col)?;
                line_text.remove(col);
                return Some(ch);
            }
        }
        None
    }

    fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}

/// Benchmark small insertions (typical typing)
fn bench_small_insertions(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_insertions");

    let text = "Hello World\nThis is a test\nOf the editor";

    group.bench_function("gap_buffer", |b| {
        b.iter(|| {
            let mut buffer = GapBuffer::from_str(text);
            for i in 0..100 {
                buffer.insert_char(i % 20, black_box('x'));
            }
            black_box(buffer.to_string());
        });
    });

    group.bench_function("vec_string", |b| {
        b.iter(|| {
            let mut buffer = VecBuffer::from_str(text);
            for i in 0..100 {
                buffer.insert_at(0, i % 20, black_box("x"));
            }
            black_box(buffer.to_string());
        });
    });

    group.finish();
}

/// Benchmark cursor movement and editing (typical editing pattern)
fn bench_cursor_editing(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor_editing");

    let text = include_str!("../Cargo.toml"); // Use a real file

    group.bench_function("gap_buffer", |b| {
        b.iter(|| {
            let mut buffer = GapBuffer::from_str(text);
            // Simulate editing at different positions
            buffer.insert_str(100, "// comment\n");
            buffer.delete_range(50..60);
            buffer.insert_char(200, '!');
            buffer.delete_char(150);
            black_box(buffer.to_string());
        });
    });

    group.bench_function("vec_string", |b| {
        b.iter(|| {
            let mut buffer = VecBuffer::from_str(text);
            // Approximate similar operations
            buffer.insert_at(2, 0, "// comment\n");
            for _ in 50..60 {
                buffer.delete_at(1, 10);
            }
            buffer.insert_at(5, 0, "!");
            buffer.delete_at(4, 0);
            black_box(buffer.to_string());
        });
    });

    group.finish();
}

/// Benchmark large file operations
fn bench_large_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_file");

    // Generate a large file (1MB)
    let large_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n".repeat(20_000);

    group.bench_function("gap_buffer_insert", |b| {
        let buffer = GapBuffer::from_str(&large_text);
        b.iter(|| {
            let mut buf = buffer.clone();
            buf.insert_str(buf.len() / 2, black_box("INSERTED TEXT"));
            black_box(buf.len());
        });
    });

    group.bench_function("vec_string_insert", |b| {
        let buffer = VecBuffer::from_str(&large_text);
        b.iter(|| {
            let mut buf = buffer.clone();
            buf.insert_at(10_000, 30, black_box("INSERTED TEXT"));
            black_box(buf.to_string().len());
        });
    });

    group.finish();
}

/// Benchmark sequential insertions (building a document)
fn bench_sequential_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_build");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("gap_buffer", size), size, |b, &size| {
            b.iter(|| {
                let mut buffer = GapBuffer::new();
                for i in 0..size {
                    buffer.insert_char(i, black_box('a'));
                }
                black_box(buffer.len());
            });
        });

        group.bench_with_input(BenchmarkId::new("vec_string", size), size, |b, &size| {
            b.iter(|| {
                let mut buffer = VecBuffer::new();
                for i in 0..size {
                    buffer.insert_at(0, i, black_box("a"));
                }
                black_box(buffer.to_string().len());
            });
        });
    }

    group.finish();
}

/// Benchmark random access patterns
fn bench_random_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_access");

    let text = "a".repeat(10000);

    group.bench_function("gap_buffer", |b| {
        let buffer = GapBuffer::from_str(&text);
        b.iter(|| {
            let mut total = 0;
            for i in [100, 5000, 2000, 8000, 1000, 9000].iter() {
                if let Some(ch) = buffer.get_char(*i) {
                    total += ch as usize;
                }
            }
            black_box(total);
        });
    });

    group.bench_function("vec_string", |b| {
        let buffer = VecBuffer::from_str(&text);
        b.iter(|| {
            let mut total = 0;
            let full_text = buffer.to_string();
            for i in [100, 5000, 2000, 8000, 1000, 9000].iter() {
                if let Some(ch) = full_text.chars().nth(*i) {
                    total += ch as usize;
                }
            }
            black_box(total);
        });
    });

    group.finish();
}

/// Benchmark line operations
fn bench_line_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_operations");

    let text = (0..1000)
        .map(|i| format!("Line {}: Some content here", i))
        .collect::<Vec<_>>()
        .join("\n");

    group.bench_function("gap_buffer_line_access", |b| {
        let buffer = GapBuffer::from_str(&text);
        b.iter(|| {
            let mut result = String::new();
            for line in [10, 50, 100, 500, 900].iter() {
                result.push_str(&buffer.get_line(*line));
            }
            black_box(result);
        });
    });

    group.bench_function("vec_string_line_access", |b| {
        let buffer = VecBuffer::from_str(&text);
        b.iter(|| {
            let mut result = String::new();
            for line in [10, 50, 100, 500, 900].iter() {
                if *line < buffer.lines.len() {
                    result.push_str(&buffer.lines[*line]);
                }
            }
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_insertions,
    bench_cursor_editing,
    bench_large_file,
    bench_sequential_build,
    bench_random_access,
    bench_line_operations
);
criterion_main!(benches);
