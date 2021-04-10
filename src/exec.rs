use std::collections::HashMap;
use std::path::Path;

use runscript::{Script, ScriptEntry, exec::CommandExecError};
use runscript::exec::{ExecConfig, exec_script};

use crate::{Options, atest::Test};

#[derive(Debug)]
pub enum DeError {
	IOError(std::io::Error),
	RecursiveError(Box<DeError>),
	RecursiveLimit,
	MismatchedTestType,
}

impl From<std::io::Error> for DeError {
    fn from(v: std::io::Error) -> Self {
        Self::IOError(v)
    }
}

pub fn run_test(test: &Test, scripts: &HashMap<String, Script>, name: &str, cwd: &Path, opts: &Options) -> TestStatus {
	let script = test.script.as_ref().map(|(name, args)| scripts.get(name).unwrap());
	let script = match script {
		Some(s) => Script {
			location: test.commands.location.clone(),
			commands: {
				let mut cmds = s.commands.clone();
				cmds.extend_from_slice(&test.commands.commands);
				cmds
			}
		},
		None => test.commands.clone()
	};
	match exec_script(&script, &ExecConfig {
		verbosity: runscript::exec::Verbosity::Silent,
		output_stream: None,
		working_directory: cwd,
		positional_args: vec![name.to_owned()],
		capture_stdout: test.xstdout.is_some() || test.xstderr.is_some(),
		env_remap: &HashMap::new(),
	}) {
		Ok(output) => {
			let exit = match test.xexit {
				Some(x) => output.status.code().map(|c| x == c).unwrap_or(false),
					// output.status.code() will only be None if terminated by a signal
				None => true
			};
			let stdout = match test.xstdout {
				Some(ref stdout) => stdout == &output.stdout,
				None => true
			};
			let stderr = match test.xstderr {
				Some(ref stderr) => stderr == &output.stderr,
				None => true
			};
			if exit && stdout && stderr {
				TestStatus::Success
			} else {
				TestStatus::Failure
			}
		},
		Err((error, c)) => TestStatus::ExecError(error, c)
	}
}

pub enum TestStatus {
	Success,
	Failure,
	ExecError(CommandExecError, ScriptEntry),
}