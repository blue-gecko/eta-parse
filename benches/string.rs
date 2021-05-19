use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(not(tarpaulin_include))]
mod strip {
    use std::borrow::Cow;

    #[inline]
    pub fn strip_with_fold(s: &str, padding: char) -> Cow<str> {
        s.chars()
            .rev()
            .skip_while(|c| *c == padding)
            .fold(String::with_capacity(s.len()), |mut s, c| {
                s.insert(0, c);
                s
            })
            .into()
    }

    #[inline]
    pub fn strip_with_array(s: &str, padding: char) -> Cow<str> {
        s.chars()
            .rev()
            .skip_while(|c| *c == padding)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
            .into()
    }

    #[inline]
    pub fn strip_with_loop(s: &str, padding: char) -> Cow<str> {
        let mut end = s.len();
        for (i, c) in s.char_indices().rev() {
            if c == padding {
                end = i;
            } else {
                break;
            }
        }
        s.get(0..end).unwrap().into()
    }
}

#[cfg(not(tarpaulin_include))]
fn strip_left_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip-left");
    for (name, input, pad, output) in [
        (
            "long",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890XXXXXXXXXXXXXXXXXX",
            'X',
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890",
        ),
        ("short", "1234567890X00", '0', "1234567890X"),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("fold", name),
            &(input, pad, output),
            |b, (input, pad, output)| {
                b.iter(|| assert_eq!(strip::strip_with_fold(input, **pad), ***output))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("array", name),
            &(input, pad, output),
            |b, (input, pad, output)| {
                b.iter(|| assert_eq!(strip::strip_with_array(input, **pad), ***output))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("loop", name),
            &(input, pad, output),
            |b, (input, pad, output)| {
                b.iter(|| assert_eq!(strip::strip_with_loop(input, **pad), ***output))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, strip_left_benchmark);
criterion_main!(benches);
