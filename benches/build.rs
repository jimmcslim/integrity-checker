extern crate integrity_checker;

#[macro_use]
extern crate criterion;

extern crate tempfile;

use std::process::Command;

use integrity_checker::database::Database;

use criterion::{Benchmark, Criterion};

use tempfile::tempdir;

fn build(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let tarball = dir.path().join("linux-4.16.7.tar.xz");
    let url = "https://cdn.kernel.org/pub/linux/kernel/v4.x/linux-4.16.7.tar.xz";
    let test_dir = dir.path().join("linux-4.16.7");

    assert!(
        Command::new("curl")
            .arg(url)
            .arg("-o")
            .arg(tarball.clone())
            .current_dir(dir.path())
            .status()
            .expect("failed to execute curl")
            .success());

    assert!(
        Command::new("tar")
            .arg("xfJ")
            .arg(tarball)
            .current_dir(dir.path())
            .status()
            .expect("failed to execute tar")
            .success());

    c.bench("build",
            Benchmark::new("linux", move |b| b.iter(|| Database::build(&test_dir, false, 1)))
            .sample_size(3));
}

criterion_group!(benches, build);
criterion_main!(benches);