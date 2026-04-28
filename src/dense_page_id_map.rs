use std::collections::HashMap;

pub type DensePageId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageEdge {
	pub from_dense_page_id: DensePageId,
	pub to_dense_page_id: DensePageId,
}
impl PageEdge {
	pub fn new(from_dense_page_id: DensePageId, to_dense_page_id: DensePageId) -> Self {
		Self {
			from_dense_page_id,
			to_dense_page_id,
		}
	}
}

pub struct DensePageIdMapBuilder {
	map: DensePageIdMap,
	next_dense_id: DensePageId,
}
impl Default for DensePageIdMapBuilder {
	fn default() -> Self {
		Self::new()
	}
}
impl DensePageIdMapBuilder {
	pub fn new() -> Self {
		Self {
			map: DensePageIdMap(HashMap::new()),
			next_dense_id: 0,
		}
	}
	pub fn push(&mut self, page_id: u32) -> DensePageId {
		let dense_id = self.next_dense_id;
		self.next_dense_id += 1;
		assert!(self.map.0.insert(page_id, dense_id).is_none());
		dense_id
	}
	pub fn build(self) -> DensePageIdMap {
		self.map
	}
}

/// maps page id to dense id
pub struct DensePageIdMap(pub HashMap<u32, DensePageId>);
impl DensePageIdMap {
	#[inline]
	pub fn get_dense_id(&self, page_id: u32) -> Option<&DensePageId> {
		self.0.get(&page_id)
	}
}
