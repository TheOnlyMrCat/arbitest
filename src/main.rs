use std::path::{Path, PathBuf};

use exec::{TestStatus, run_test};
use runscript::parser::RunscriptSource;

mod atest;
mod exec;
mod parser;

use atest::Arbitest;
use parser::parse_arbitest;

fn main() {
	let filename = std::env::args().nth(1).unwrap_or("atest".to_owned());
	let filepath = PathBuf::from(filename.clone()).canonicalize().unwrap();

	let options = Options::default();
	let source = RunscriptSource {
		file: filepath.clone(),
		base: filepath.parent().expect("File to be a file").to_owned(),
		index: vec![],
		source: std::fs::read_to_string(&filepath).unwrap(),
	};
	
    match parse_arbitest(source) {
		Ok(config) => {
			execute_config(&config, &filename, &filepath.parent().expect("Checked above"), &options);
		},
		Err(_) => todo!(),
	}
}

fn execute_config(config: &Arbitest, filename: &str, cwd: &Path, opts: &Options) -> bool {
	eprintln!("Running {} test{s} in {}:", config.tests.len(), filename, s = if config.tests.len() == 1 { "" } else { "s" });

	let mut pass_count = 0;
	let mut fail_count = 0;
	for (name, test) in &config.tests {
		eprint!("    {}...", &name);
		match run_test(test, &config.scripts, name, cwd, &opts) {
			TestStatus::Success => { eprintln!("PASS"); pass_count += 1 },
			TestStatus::Failure => { eprintln!("FAIL"); fail_count += 1 },
			TestStatus::ExecError(e, c) => panic!("{:?}", e)
		}
	}
	
	eprintln!("test_results: {} ({} passed, {} failed)", if fail_count == 0 { "ok" } else { "failed" }, pass_count, fail_count);
	true
}

pub struct Options {
	max_depth: u32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
			max_depth: 1000,
		}
    }
}