mod exec;
mod test;
use std::path::PathBuf;

use exec::{DeError, TestStatus, deserialize_validate_config, run_test};
use test::{Action, ArbitestConfig};

fn main() {
	let options = Options::default();
    match deserialize_validate_config("Arbitest.toml", &options, 0) {
		Ok(config) => {
			execute_config(&config, "Arbitest.toml", &options);
		},
		Err(DeError::RecursiveLimit) => {
			eprintln!("atest: reached recursive config limit");
		},
		Err(_) => todo!(),
	}
}

fn execute_config(config: &ArbitestConfig, filename: &str, opts: &Options) -> bool {
	if !config.config.delegate {
		println!("Running {} tests in {}:", config.tests.len(), filename);
	}

	let mut pass_count = 0;
	let mut fail_count = 0;
	for test in &config.tests {
		if matches!(&test.action, Action::Command(s) if s == "subtest") {
			execute_config(test.subtest.as_ref().expect("added in validation").sub_config.as_ref().expect("added in validation"), &test.file, opts);
		} else {
			print!("    {}...", &test.file);
			match run_test(test, &config.commands, PathBuf::from(filename).canonicalize().unwrap().parent().unwrap(), &opts) {
				TestStatus::Success => { println!("PASS"); pass_count += 1 },
				TestStatus::Failure => { println!("FAIL"); fail_count += 1 },
				TestStatus::IOError(e) => panic!("{}", e)
			}
		}
	}

	if !config.config.delegate {
		println!("test_results: {} ({} passed, {} failed)", if fail_count == 0 { "ok" } else { "failed" }, pass_count, fail_count);
	}

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