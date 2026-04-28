use std::{
	fs::File,
	io::{self, BufRead, BufReader},
	path::Path,
};

/// Parse all INSERT rows from a Wikipedia SQL dump line by line.
/// Calls `on_row` for every `(col1, col2, ...)` tuple found.
///
/// Wikipedia dumps use the form:
/// ```sql
/// INSERT INTO `table` VALUES (v1,v2,...),(v3,v4,...);
/// ```
pub fn parse_sql_inserts<F>(path: &Path, mut on_row: F) -> io::Result<u64>
where
	F: FnMut(&[SqlValue]) -> io::Result<()>,
{
	let file = File::open(path)?;
	let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
	let mut count = 0u64;

	for line in reader.lines() {
		let line = line?;
		let line = line.trim();
		if !line.starts_with("INSERT INTO") {
			continue;
		}
		// Find VALUES keyword
		let Some(values_pos) = line.find("VALUES") else {
			continue;
		};
		let data = &line[values_pos + 6..].trim_start();
		// Parse tuples
		let mut chars = data.chars().peekable();
		loop {
			// skip whitespace/commas between tuples
			while chars.peek() == Some(&',') || chars.peek() == Some(&' ') {
				chars.next();
			}
			if chars.peek() != Some(&'(') {
				break;
			}
			chars.next(); // consume '('
			let cols = parse_row(&mut chars);
			on_row(&cols)?;
			count += 1;
			// expect ')' already consumed by parse_row
		}
	}
	Ok(count)
}

#[derive(Debug, Clone)]
pub enum SqlValue {
	Null,
	Integer(i64),
	UnsignedInt(u64),
	Float(f64),
	Bytes(Vec<u8>),
}

impl SqlValue {
	pub fn as_u32(&self) -> Option<u32> {
		match self {
			SqlValue::Integer(v) => Some(*v as u32),
			SqlValue::UnsignedInt(v) => Some(*v as u32),
			_ => None,
		}
	}
	pub fn as_i32(&self) -> Option<i32> {
		match self {
			SqlValue::Integer(v) => Some(*v as i32),
			SqlValue::UnsignedInt(v) => Some(*v as i32),
			_ => None,
		}
	}
	pub fn as_u64(&self) -> Option<u64> {
		match self {
			SqlValue::Integer(v) => Some(*v as u64),
			SqlValue::UnsignedInt(v) => Some(*v),
			_ => None,
		}
	}
	pub fn as_u8(&self) -> Option<u8> {
		match self {
			SqlValue::Integer(v) => Some(*v as u8),
			SqlValue::UnsignedInt(v) => Some(*v as u8),
			_ => None,
		}
	}
	pub fn as_bytes(&self) -> Option<&[u8]> {
		match self {
			SqlValue::Bytes(b) => Some(b),
			_ => None,
		}
	}
}

/// Parse a single row's values until the closing `)`, consuming it.
/// Handles: NULL, integers, floats, single-quoted strings with `\'` and `\\`
/// escapes, and hex literals like `0x...`.
fn parse_row(chars: &mut std::iter::Peekable<std::str::Chars>) -> Vec<SqlValue> {
	let mut cols = Vec::new();
	loop {
		// skip leading whitespace
		while chars.peek() == Some(&' ') {
			chars.next();
		}
		match chars.peek() {
			None | Some(&')') => {
				chars.next(); // consume ')'
				break;
			}
			Some(&',') => {
				chars.next();
			}
			Some(&'\'') => {
				chars.next();
				cols.push(parse_string(chars));
			}
			Some(&'N') => {
				// NULL
				for _ in 0..4 {
					chars.next();
				}
				cols.push(SqlValue::Null);
			}
			Some(&'0') => {
				// Could be 0x hex or number
				let s = collect_token(chars);
				if s.starts_with("0x") || s.starts_with("0X") {
					let hex = &s[2..];
					let v = u64::from_str_radix(hex, 16).unwrap_or(0);
					cols.push(SqlValue::UnsignedInt(v));
				} else {
					cols.push(parse_number_token(&s));
				}
			}
			Some(&c) if c == '-' || c.is_ascii_digit() => {
				let s = collect_token(chars);
				cols.push(parse_number_token(&s));
			}
			_ => {
				// skip unknown token
				collect_token(chars);
			}
		}
	}
	cols
}

fn collect_token(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
	let mut s = String::new();
	while let Some(&c) = chars.peek() {
		if c == ',' || c == ')' || c == ' ' {
			break;
		}
		s.push(c);
		chars.next();
	}
	s
}

fn parse_number_token(s: &str) -> SqlValue {
	if s.contains('.')
		&& let Ok(f) = s.parse::<f64>()
	{
		return SqlValue::Float(f);
	}
	if let Ok(v) = s.parse::<u64>() {
		return SqlValue::UnsignedInt(v);
	}
	if let Ok(v) = s.parse::<i64>() {
		return SqlValue::Integer(v);
	}
	SqlValue::Bytes(s.as_bytes().to_vec())
}

/// Parse a single-quoted MySQL string. The opening `'` has already been consumed.
/// Handles `\'`, `\\`, `\n`, `\r`, `\t`, `\0`, `\Z` escapes.
/// The closing `'` is consumed.
fn parse_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> SqlValue {
	let mut bytes: Vec<u8> = Vec::new();
	loop {
		match chars.next() {
			None => break,
			Some('\'') => break,
			Some('\\') => {
				match chars.next() {
					Some('\'') => bytes.push(b'\''),
					Some('\\') => bytes.push(b'\\'),
					Some('n') => bytes.push(b'\n'),
					Some('r') => bytes.push(b'\r'),
					Some('t') => bytes.push(b'\t'),
					Some('0') => bytes.push(b'\0'),
					Some('Z') => bytes.push(0x1A),
					Some(c) => {
						// encode as UTF-8
						let mut buf = [0u8; 4];
						bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
					}
					None => break,
				}
			}
			Some(c) => {
				let mut buf = [0u8; 4];
				bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
			}
		}
	}
	SqlValue::Bytes(bytes)
}
