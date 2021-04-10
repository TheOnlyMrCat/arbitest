use std::collections::HashMap;

use runscript::{ArgPart, Argument, ChainedCommand, Script, ScriptEntry};
use runscript::parser::{ParsingContext, RunscriptSource, RunscriptParseError, RunscriptParseErrorData};
use runscript::parser::{BreakCondition, consume_line, consume_word};
use runscript::parser::{parse_commands, parse_command};

use crate::atest::{Arbitest, Test};

pub fn parse_arbitest(source: RunscriptSource) -> Result<Arbitest, RunscriptParseError> {
	let mut context = ParsingContext::new(&source);
	parse_root(&mut context).map_err(|data| RunscriptParseError { script: context.runfile, data })
}

fn parse_root<T: Iterator<Item = (usize, char)> + std::fmt::Debug>(context: &mut ParsingContext<T>) -> Result<Arbitest, RunscriptParseErrorData> {
	let mut atest = Arbitest {
		scripts: HashMap::new(),
		tests: HashMap::new(),
	};

	while let Some(tk) = context.iterator.next() { match tk {
		// Comment
		(_, '!') => {
			consume_line(&mut context.iterator);
		},
		// Script
		(i, '#') => {
			let (name, bk) = consume_word(&mut context.iterator);

			if name == "#" {
				// Parse test
				let (mut name, bk) = consume_word(&mut context.iterator);

				let script = if name.chars().last().unwrap()/*TODO*/ == ':' {
					name.pop();
					match parse_command(context, ChainedCommand::None, None)? {
					    Some(ScriptEntry::Command(c)) if matches!(*c.chained, ChainedCommand::None) => Some((
							if let Argument::Unquoted(ArgPart::Str(s)) = *c.target {
								s
							} else {
								todo!()
							},
							c.args
						)),
					    g =>  todo!(),
					}
				} else {
					None
				};

				enum NextSegment {
					Finished(Option<i32>), // #/[:digit:]*
					ExpectedStreamOutput(u32) // #[01]
				}

				let (commands, mut next_seg) = {
					let mut commands = Vec::new();
					let next = loop {
						match context.iterator.peek() {
							Some((_, '#')) => {
								context.iterator.next();
								match context.iterator.next() {
									Some((_, c)) if c.is_ascii_digit() => {
										consume_line(&mut context.iterator);
										break NextSegment::ExpectedStreamOutput(c as u32 - '0' as u32);
									},
									Some((_, '/')) => {
										match context.iterator.next() {
											Some((_, '\n')) | None => break NextSegment::Finished(Some(0)),
											Some((_, '-')) => break NextSegment::Finished(None),
											Some((_, c)) if c.is_ascii_digit() => {
												let mut s = String::new();
												s.push(c);
												let (n, bk) = consume_word(&mut context.iterator);
												s.push_str(&n);
												let x = s.parse::<i32>().unwrap();
												break NextSegment::Finished(Some(x));
											},
											_ => todo!(),
										}
									},
									_ => todo!(),
								}
							},
							_ => {
								commands.push(parse_command(context, ChainedCommand::None, None)?.expect("/#.*/ was checked for above"))
							}
						}
					};
					(commands, next)
				};

				let mut test = Test {
				    script,
				    commands: Script { commands, location: context.get_loc(i) },
				    xexit: None,
				    xstdout: None,
				    xstderr: None,
				};

				loop {
					match next_seg {
						NextSegment::ExpectedStreamOutput(i) => {
							let mut text = Vec::new();
							next_seg = loop {
								match context.iterator.peek() {
									Some((_, '#')) => {
										context.iterator.next();
										match context.iterator.next() {
											Some((_, c)) if c.is_ascii_digit() => {
												consume_line(&mut context.iterator);
												break NextSegment::ExpectedStreamOutput(c as u32 - '0' as u32);
											},
											Some((_, '/')) => {
												match context.iterator.next() {
													Some((_, '\n')) => break NextSegment::Finished(Some(0)),
													Some((_, '-')) => break NextSegment::Finished(None),
													Some((_, c)) if c.is_ascii_digit() => {
														let mut s = String::new();
														s.push(c);
														let (n, bk) = consume_word(&mut context.iterator);
														s.push_str(&n);
														let x = s.parse::<i32>().unwrap();
														break NextSegment::Finished(Some(x));
													},
													_ => todo!(),
												}
											},
											_ => todo!(),
										}
									},
									_ => {
										if !text.is_empty() {
											text.push('\n' as u8);
										}
										text.extend(consume_line(&mut context.iterator).0.bytes());
									}
								}
							};
							match i {
								0 => test.xstdout = Some(text),
								1 => test.xstderr = Some(text),
								_ => todo!(),
							}
						},
						NextSegment::Finished(st) => {
							test.xexit = st;
							break;
						}
					}
				}

				atest.tests.insert(name, test);
			} else {
				// Parse script
				if !matches!(bk, BreakCondition::Newline(_)) {
					consume_line(&mut context.iterator);
				}

				let script = Script {
					location: context.get_loc(i),
					commands: parse_commands(context)?,
				};

				if name.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-')) {
					return Err(RunscriptParseErrorData::InvalidID { location: context.get_loc(i + 1), found: name });
				}

				if let Some(prev_script) = atest.scripts.insert(name.clone(), script) {
					return Err(RunscriptParseErrorData::MultipleDefinition {
						previous_location: prev_script.location.clone(),
						new_location: context.get_loc(i),
						target_name: name,
					});
				}
			}
		},
		(_, c) if c.is_whitespace() => {},
		_ => todo!(),
	}}

	Ok(atest)
}