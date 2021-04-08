use std::convert::TryInto;

use serde::{Deserialize, de::{Error, Unexpected, Visitor}};

#[derive(Debug, Default, Deserialize)]
pub struct ArbitestConfig {
	#[serde(default)]
	pub config: Config,
	pub tests: Vec<Test>,
	#[serde(default)]
	pub commands: Vec<Command>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
	pub delegate: bool,
}

#[derive(Debug, Deserialize)]
pub struct Test {
	pub file: String,
	pub action: Action,
	#[serde(default)]
	pub xout: XOut,
	
	#[serde(default)]
	pub command: Option<CommandConfig>,
	#[serde(default)]
	pub subtest: Option<SubTestConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SubTestConfig {
	#[serde(skip)]
	pub sub_config: Option<Box<ArbitestConfig>>,
	pub breakdown: bool,
}

impl Default for SubTestConfig {
	fn default() -> Self {
        SubTestConfig { breakdown: true, sub_config: None }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct CommandConfig {
	pub args: Vec<String>,
}

#[derive(Debug)]
pub enum Action {
	CommandRef(usize),
	Command(String),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct XOut {
	pub code: Option<i32>,
	pub stdout: Option<Vec<u8>>,
	pub stderr: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct Command {
	pub command: String,
    #[serde(default)]
	pub args: Vec<String>,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
	{	
		struct StringOrInt;

		impl<'de> Visitor<'de> for StringOrInt {
			type Value = Action;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a string or an unsigned 32-bit integer")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where E: Error,
			{
				Ok(Action::Command(v.to_owned()))
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where E: Error,
			{
				Ok(Action::Command(v))
			}

			fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
			where E: Error,
			{
				Ok(Action::CommandRef(v.try_into().map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))?))
			}

			fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
			where E: Error,
			{
				Ok(Action::CommandRef(v.try_into().map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))?))
			}
		}

		deserializer.deserialize_any(StringOrInt)
    }
}