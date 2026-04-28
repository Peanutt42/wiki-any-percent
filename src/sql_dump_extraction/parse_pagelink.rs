use crate::sql_dump_extraction::{PageLink, parse_sql_inserts};
use std::path::PathBuf;

pub fn parse_pagelink_sql_dump(input_filepath: PathBuf, mut callback: impl FnMut(PageLink)) {
	parse_sql_inserts(&input_filepath, |cols| {
		if cols.len() < 3 {
			return Ok(());
		}
		let Some(pl_from) = cols[0].as_u32() else {
			return Ok(());
		};
		let Some(ns) = cols[1].as_i32() else {
			return Ok(());
		};
		let Some(pl_target_id) = cols[2].as_u64() else {
			return Ok(());
		};

		if ns != 0 {
			return Ok(());
		}

		callback(PageLink {
			pl_from,
			pl_from_namespace: ns,
			pl_target_id,
		});

		Ok(())
	})
	.unwrap();
}
