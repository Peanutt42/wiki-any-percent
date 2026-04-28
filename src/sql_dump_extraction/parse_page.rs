use std::path::Path;

use crate::sql_dump_extraction::{Page, parse_sql_inserts};

pub fn parse_page_sql_dump(input: &Path, mut callback: impl FnMut(Page)) {
	let ns_filter = Some(0);
	let redirects_only = false;

	parse_sql_inserts(input, |cols| {
		// Schema (12 columns):
		//  0: page_id
		//  1: page_namespace
		//  2: page_title
		//  3: page_is_redirect
		//  4: page_is_new
		//  5: page_random
		//  6: page_touched
		//  7: page_links_updated
		//  8: page_latest
		//  9: page_len
		// 10: page_content_model
		// 11: page_lang
		if cols.len() < 4 {
			return Ok(());
		}
		let Some(page_id) = cols[0].as_u32() else {
			return Ok(());
		};
		let Some(page_namespace) = cols[1].as_i32() else {
			return Ok(());
		};
		let Some(page_title) = cols[2].as_bytes() else {
			return Ok(());
		};
		let page_is_redirect = cols[3].as_u8().unwrap_or(0) != 0;

		if redirects_only && !page_is_redirect {
			return Ok(());
		}
		if let Some(required_ns) = ns_filter
			&& page_namespace != required_ns
		{
			return Ok(());
		}

		callback(Page {
			page_id,
			page_namespace,
			page_is_redirect,
			page_title: page_title.to_vec(),
		});

		Ok(())
	})
	.unwrap();
}
