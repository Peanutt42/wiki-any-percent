use std::path::PathBuf;

use crate::sql_dump_extraction::{LinkTarget, parse_sql_inserts};

pub fn parse_linktarget_sql_dump(input_filepath: PathBuf, mut callback: impl FnMut(LinkTarget)) {
	let ns_filter = Some(0);

	parse_sql_inserts(&input_filepath, |cols| {
		// Schema: lt_id, lt_namespace, lt_title
		if cols.len() < 3 {
			return Ok(());
		}
		let Some(lt_id) = cols[0].as_u64() else {
			return Ok(());
		};
		let Some(lt_namespace) = cols[1].as_i32() else {
			return Ok(());
		};
		let Some(lt_title) = cols[2].as_bytes() else {
			return Ok(());
		};

		if let Some(required_ns) = ns_filter
			&& lt_namespace != required_ns
		{
			return Ok(());
		}

		callback(LinkTarget {
			lt_id,
			lt_namespace,
			lt_title: lt_title.to_vec(),
		});

		Ok(())
	})
	.unwrap();
}
