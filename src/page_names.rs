use rkyv::{Archive, Deserialize, Serialize};
use std::path::Path;

use crate::DensePageId;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub struct PageNameToDensePageIdEntry {
	pub name: String,
	pub dense_page_id: DensePageId,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub struct PageNames {
	/// sorted by the entries name for binary search using the name as key
	page_name_to_dense_page_id_entries: Vec<PageNameToDensePageIdEntry>,
	/// maps dense page id to the index of its page name inside `page_name_to_dense_page_id_entries`
	dense_page_id_to_page_name_indicies: Vec<Option<u32>>,
}
impl PageNames {
	/// sorts entires by name for binary search
	pub fn build(mut page_name_to_dense_page_id_entries: Vec<PageNameToDensePageIdEntry>) -> Self {
		page_name_to_dense_page_id_entries.sort_unstable_by_key(|e| e.name.clone());
		let mut dense_page_id_to_page_name_indicies =
			vec![None; page_name_to_dense_page_id_entries.len()];
		for (i, entry) in page_name_to_dense_page_id_entries.iter().enumerate() {
			dense_page_id_to_page_name_indicies[entry.dense_page_id as usize] = Some(i as u32);
		}
		Self {
			page_name_to_dense_page_id_entries,
			dense_page_id_to_page_name_indicies,
		}
	}

	pub fn save(&self, filepath: impl AsRef<Path>) {
		std::fs::write(
			filepath,
			rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap(),
		)
		.unwrap();
	}
	pub fn lookup_name(&self, name: &str) -> Option<DensePageId> {
		let index = self
			.page_name_to_dense_page_id_entries
			.binary_search_by_key(&name, |e| &e.name)
			.ok()?;
		self.page_name_to_dense_page_id_entries
			.get(index)
			.map(|e| e.dense_page_id)
	}

	pub fn lookup_dense_page_id(&self, dense_page_id: DensePageId) -> Option<&str> {
		let page_name_entry_index = (*self
			.dense_page_id_to_page_name_indicies
			.get(dense_page_id as usize)?)?;
		self.page_name_to_dense_page_id_entries
			.get(page_name_entry_index as usize)
			.map(|e| e.name.as_str())
	}
}
impl ArchivedPageNames {
	pub fn lookup_name(&self, name: &str) -> Option<DensePageId> {
		let index = self
			.page_name_to_dense_page_id_entries
			.binary_search_by_key(&name, |e| &e.name)
			.ok()?;
		self.page_name_to_dense_page_id_entries
			.get(index)
			.map(|e| e.dense_page_id.to_native())
	}
	pub fn lookup_dense_page_id(&self, dense_page_id: DensePageId) -> Option<&str> {
		let page_name_entry_index = self
			.dense_page_id_to_page_name_indicies
			.get(dense_page_id as usize)?
			.as_ref()?
			.to_native();
		self.page_name_to_dense_page_id_entries
			.get(page_name_entry_index as usize)
			.map(|e| e.name.as_str())
	}
	pub fn num_pages(&self) -> usize {
		self.page_name_to_dense_page_id_entries.len()
	}
}
