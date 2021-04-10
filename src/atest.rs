use std::collections::HashMap;

use runscript::{Argument, Script, ScriptEntry};

#[derive(Debug)]
pub struct Arbitest {
	pub scripts: HashMap<String, Script>,
	pub tests: HashMap<String, Test>,
}

#[derive(Debug)]
pub struct Test {
	pub script: Option<(String, Vec<Argument>)>,
	pub commands: Script,
	pub xexit: Option<i32>,
	pub xstdout: Option<Vec<u8>>,
	pub xstderr: Option<Vec<u8>>,
}