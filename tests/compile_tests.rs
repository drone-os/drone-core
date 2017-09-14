extern crate compiletest_rs as compiletest;
extern crate glob;

use glob::glob;
use std::fs;
use std::path::PathBuf;

fn run_mode(mode: &'static str) {
  let mut config = compiletest::Config::default().tempdir();
  config.mode = mode.parse().expect("Invalid mode");
  config.src_base = PathBuf::from(format!("tests/{}", mode));
  config.target_rustcflags = Some("-L target/debug/deps".to_string());
  compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
  for entry in glob("target/debug/deps/*.rmeta").unwrap() {
    if let Ok(path) = entry {
      fs::remove_file(path).unwrap();
    }
  }
  run_mode("compile-fail");
  run_mode("run-pass");
}
