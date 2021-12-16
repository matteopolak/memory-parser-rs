#![feature(map_try_insert)]

use std::env;
use std::fs;
use std::collections::{HashMap, HashSet};
use regex::Regex;

static REGEX_EXPRESSION: &str = "((?:\\(\\?i\\))?\\^[ -~]{3,})(?:\0|<|\\\\s\\*)|\0(/[-\\w/\\.\\s]{3,})\0|\0\0\0([A-Z_]{2,})\0\0\0|(cff9)|(a238)";

#[derive(Debug)]
struct Match<'a> {
	expr: &'a str,
	is_needed: bool
}

#[derive(Debug)]
struct FileEntry<'a> {
	matchers: Vec<Match<'a>>,
	category: &'a str,
	flag: &'a str
}

#[derive(Debug)]
struct FlagEntry<'a> {
	matchers: Vec<Match<'a>>,
	category: &'a str,
	files: Vec<&'a str>
}

fn populate_set(mut set: HashSet<&str>) -> HashSet<&str> {
	set.insert("FOR");
	set.insert("USR");
	set.insert("ACT");
	set.insert("POL");
	set.insert("DEF");
	set.insert("SRV");
	set.insert("OUP");
	set.insert("AUP");
	set.insert("FIL");
	set.insert("SFT");
	set.insert("APP");
	set.insert("PEN");
	set.insert("SCR");
	set.insert("SYS");
	set.insert("MAL");

	set
}

fn main() {
	let data = match env::args().skip(1).next() {
		Some(p) => {
			fs::read(p).unwrap()
		},
		None => std::process::exit(-1)
	};

	let parsed = String::from_utf8_lossy(&data);
	let expression = Regex::new(REGEX_EXPRESSION).unwrap();

	let categories: HashSet<&str> = {
		let set = HashSet::new();

		populate_set(set)
	};
	
	let mut by_file_name: HashMap<&str, FileEntry> = HashMap::new();
	let mut by_flag_name: HashMap<&str, FlagEntry> = HashMap::new();

	let mut current_file = "_UNKNOWN";
	let mut current_flag = "_UNKNOWN";
	let mut current_category = "_UNKNOWN";
	let mut expr_required = true;

	by_file_name.insert(current_file, FileEntry {
		matchers: vec!(),
		category: current_category,
		flag: current_flag
	});

	by_flag_name.insert(current_file, FlagEntry {
		matchers: vec!(),
		category: current_category,
		files: vec!()
	});

	for content in expression.captures_iter(&parsed) {
		match content.iter().skip(1).position(|m| m.is_some()).unwrap_or(999) {
			0 => {
				// regex
				let text = content.get(1).unwrap().as_str();

				by_flag_name.get_mut(current_flag).unwrap().matchers.push(Match {
					expr: text,
					is_needed: expr_required
				});

				by_file_name.get_mut(current_file).unwrap().matchers.push(Match {
					expr: text,
					is_needed: expr_required
				});
			},
			1 => {
				// file
				let text = content.get(2).unwrap().as_str();

				current_file = text;
				by_flag_name.get_mut(current_flag).unwrap().files.push(text);
				
				let file_entry = FileEntry {
					matchers: vec!(),
					category: current_category,
					flag: current_flag
				};

				by_file_name.try_insert(current_file, file_entry).ok();
			},
			2 => {
				let text = content.get(3).unwrap().as_str();
				
				let file_entry = FileEntry {
					matchers: vec!(),
					category: current_category,
					flag: current_flag
				};

				match categories.contains(text) {
					true => {
						current_category = text;

						by_file_name.get_mut(current_file).unwrap().category = current_category;
						by_flag_name.get_mut(current_flag).unwrap().category = current_category;
					},
					false => {
						let flag_entry = FlagEntry {
							matchers: vec!(),
							category: current_category,
							files: vec!()
						};

						current_flag = text;
						by_flag_name.try_insert(current_flag, flag_entry).ok();

						match by_file_name.try_insert(current_file, file_entry) {
							Ok(_) => (),
							Err(doc) => {
								doc.entry.into_mut().flag = current_flag;
							}
						}
					}
				}
			},
			3 => expr_required = true,
			4 => expr_required = false,
			_ => continue
		};
	}

	fs::write("./by_flag.txt", format!("{:#?}", by_flag_name)).ok();
	fs::write("./by_file.txt", format!("{:#?}", by_file_name)).ok();
}