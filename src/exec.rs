use std::{path::{Path, PathBuf}, process::{Command, ExitStatus, Stdio}};

use serde::Deserialize;

use crate::{Options, test::{Action, ArbitestConfig, Test}};

pub fn deserialize_validate_config(path: impl AsRef<Path>, opts: &Options, depth: u32) -> Result<ArbitestConfig, DeError> {
	if depth > opts.max_depth {
		Err(DeError::RecursiveLimit)?
	}

	let source = std::fs::read_to_string(path)?;
	let mut deser = toml::Deserializer::new(&source);
	let mut config = ArbitestConfig::deserialize(&mut deser)?;
	deser.end()?;

	for test in &mut config.tests {
		match &test.action {
			Action::CommandRef(_i) => {
				
			},
			Action::Command(s) => {
				if s == "subtest" {
					let subtest = test.subtest.get_or_insert_with(Default::default);
					subtest.sub_config = Some(Box::new(match deserialize_validate_config(&test.file, opts, depth + 1) {
						Ok(cfg) => cfg,
						Err(DeError::RecursiveLimit) => Err(DeError::RecursiveLimit)?,
						Err(e) => Err(DeError::RecursiveError(Box::new(e)))?,
					}));
				} else {
					if test.subtest.is_some() {
						Err(DeError::MismatchedTestType)?
					}
				}
			}
		}
	}

	Ok(config)
}

#[derive(Debug)]
pub enum DeError {
	IOError(std::io::Error),
	Format(toml::de::Error),
	RecursiveError(Box<DeError>),
	RecursiveLimit,
	MismatchedTestType,
}

impl From<std::io::Error> for DeError {
    fn from(v: std::io::Error) -> Self {
        Self::IOError(v)
    }
}

impl From<toml::de::Error> for DeError {
    fn from(v: toml::de::Error) -> Self {
        Self::Format(v)
    }
}

pub fn run_test(test: &Test, default_commands: &[crate::test::Command], cwd: &Path, opts: &Options) -> TestStatus {
	let (cmd, args) = match &test.action {
		Action::Command(cmd) => {
			(cmd, test.command.as_ref().map(|c| c.args.as_slice()).unwrap_or(&[]))
		},
		Action::CommandRef(i) => {
			(&default_commands[*i].command, default_commands[*i].args.as_slice())
		}
	};
	let child = Command::new(cmd)
		.args(args.into_iter().map(|s| if s == "$FILE" { &test.file } else { s }))
		.current_dir(cwd)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn();

	let mut child = match child {
		Ok(child) => child,
		Err(e) => return e.into(),
	};
	
	match child.wait() {
		Ok(st) => {
			match test.xout.code {
				Some(code) => if st.code().map_or(false, |c| c == code) {
						TestStatus::Success
					} else {
						TestStatus::Failure
					},
				None => if st.success() {
						TestStatus::Success
					} else {
						TestStatus::Failure
					}
			}
		},
		Err(e) => e.into(),
	}
}

pub enum TestStatus {
	Success,
	Failure,
	IOError(std::io::Error),
}

impl From<std::io::Error> for TestStatus {
    fn from(v: std::io::Error) -> Self {
        Self::IOError(v)
    }
}