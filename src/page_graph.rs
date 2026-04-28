use rkyv::{Archive, Deserialize, Serialize};
use std::path::Path;

use crate::{DensePageId, PageEdge, PageRank};

#[derive(Archive, Serialize, Deserialize)]
pub struct PageGraph {
	pub forward_offsets: Vec<u32>,
	pub forward_edges: Vec<DensePageId>,

	pub backward_offsets: Vec<u32>,
	pub backward_edges: Vec<DensePageId>,
}
impl PageGraph {
	pub fn build<'a>(
		page_edges_iter: impl Iterator<Item = &'a PageEdge>,
		page_edges_iter2: impl Iterator<Item = &'a PageEdge>,
		num_pages: usize,
	) -> Self {
		let mut forward_degree = vec![0u32; num_pages];
		let mut backward_degree = vec![0u32; num_pages];

		for edge in page_edges_iter {
			forward_degree[edge.from_dense_page_id as usize] += 1;
			backward_degree[edge.to_dense_page_id as usize] += 1;
		}

		let mut forward_offsets = vec![0u32; num_pages + 1];
		let mut backward_offsets = vec![0u32; num_pages + 1];

		for i in 0..num_pages {
			forward_offsets[i + 1] = forward_offsets[i] + forward_degree[i];
			backward_offsets[i + 1] = backward_offsets[i] + backward_degree[i];
		}

		let forward_num_edges = forward_offsets[num_pages] as usize;
		let mut forward_edges = vec![0u32; forward_num_edges];
		let mut forward_cursor = forward_offsets.clone();

		let backward_num_edges = forward_offsets[num_pages] as usize;
		let mut backward_edges = vec![0u32; backward_num_edges];
		let mut backward_cursor = backward_offsets.clone();

		for edge in page_edges_iter2 {
			let forward_pos = forward_cursor[edge.from_dense_page_id as usize];
			forward_edges[forward_pos as usize] = edge.to_dense_page_id;
			forward_cursor[edge.from_dense_page_id as usize] += 1;

			let backward_pos = backward_cursor[edge.to_dense_page_id as usize];
			backward_edges[backward_pos as usize] = edge.from_dense_page_id;
			backward_cursor[edge.to_dense_page_id as usize] += 1;
		}

		let mut this = Self {
			forward_offsets,
			forward_edges,
			backward_edges,
			backward_offsets,
		};

		let page_rank = PageRank::build(&this, num_pages);
		let page_rank = page_rank.get_rank();

		let sort_by_pagerank_cmp = |linked_dense_page_id_a: &u32, linked_dense_page_id_b: &u32| {
			page_rank[*linked_dense_page_id_b as usize]
				.total_cmp(&page_rank[*linked_dense_page_id_a as usize])
		};

		for dense_page_id in 0..num_pages {
			// sort by pagerank
			this.internal_forward_linked_pages_mut(dense_page_id)
				.sort_unstable_by(sort_by_pagerank_cmp);

			// sort adjacency lists
			this.internal_backward_linked_pages_mut(dense_page_id)
				.sort_unstable();
		}

		this
	}

	pub fn save(&self, filepath: impl AsRef<Path>) {
		std::fs::write(
			filepath,
			rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap(),
		)
		.unwrap();
	}

	fn internal_forward_linked_pages(&self, dense_page_id: usize) -> &[u32] {
		&self.forward_edges[self.forward_offsets[dense_page_id] as usize
			..self.forward_offsets[dense_page_id + 1] as usize]
	}
	fn internal_forward_linked_pages_mut(&mut self, dense_page_id: usize) -> &mut [u32] {
		&mut self.forward_edges[self.forward_offsets[dense_page_id] as usize
			..self.forward_offsets[dense_page_id + 1] as usize]
	}
	pub fn iter_forward_linked_pages(
		&self,
		dense_page_id: DensePageId,
	) -> std::slice::Iter<'_, DensePageId> {
		self.internal_forward_linked_pages(dense_page_id as usize)
			.iter()
	}

	fn internal_backward_linked_pages(&self, dense_page_id: usize) -> &[u32] {
		&self.backward_edges[self.backward_offsets[dense_page_id] as usize
			..self.backward_offsets[dense_page_id + 1] as usize]
	}
	fn internal_backward_linked_pages_mut(&mut self, dense_page_id: usize) -> &mut [u32] {
		&mut self.backward_edges[self.backward_offsets[dense_page_id] as usize
			..self.backward_offsets[dense_page_id + 1] as usize]
	}
	pub fn iter_backward_linked_pages(
		&self,
		dense_page_id: DensePageId,
	) -> std::slice::Iter<'_, DensePageId> {
		self.internal_backward_linked_pages(dense_page_id as usize)
			.iter()
	}
}

impl ArchivedPageGraph {
	pub fn iter_forward_linked_pages(
		&self,
		dense_page_id: DensePageId,
	) -> std::slice::Iter<'_, rkyv::rend::u32_le> {
		self.forward_edges[self.forward_offsets[dense_page_id as usize].to_native() as usize
			..self.forward_offsets[dense_page_id as usize + 1].to_native() as usize]
			.iter()
	}

	pub fn iter_backward_linked_pages(
		&self,
		dense_page_id: DensePageId,
	) -> std::slice::Iter<'_, rkyv::rend::u32_le> {
		self.backward_edges[self.backward_offsets[dense_page_id as usize].to_native() as usize
			..self.backward_offsets[dense_page_id as usize + 1].to_native() as usize]
			.iter()
	}

	pub fn num_edges(&self) -> usize {
		self.forward_edges.len()
	}
}
