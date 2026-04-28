mod parse_linktarget;
pub use parse_linktarget::parse_linktarget_sql_dump;

mod parse_page;
pub use parse_page::parse_page_sql_dump;

mod parse_pagelink;
pub use parse_pagelink::parse_pagelink_sql_dump;

mod parse_sql;
pub use parse_sql::parse_sql_inserts;

#[derive(Debug, Clone, PartialEq)]
pub struct PageLink {
	pub pl_from: u32,
	pub pl_from_namespace: i32,
	pub pl_target_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkTarget {
	pub lt_id: u64,
	pub lt_namespace: i32,
	pub lt_title: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
	pub page_id: u32,
	pub page_namespace: i32,
	pub page_is_redirect: bool,
	pub page_title: Vec<u8>,
}
