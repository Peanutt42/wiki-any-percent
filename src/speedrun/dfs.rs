use crate::{ArchivedPageGraph, DensePageId};
use std::collections::{HashSet, VecDeque};

pub fn speedrun_any_percent_dfs(
	start_dense_page_id: DensePageId,
	end_dense_page_id: DensePageId,
	page_graph: &ArchivedPageGraph,
) -> Option<Vec<DensePageId>> {
	let mut current_dense_page_id = start_dense_page_id;
	let mut visited_dense_page_ids = HashSet::new();
	let mut visited_dense_page_id_trail: VecDeque<DensePageId> = VecDeque::new();
	visited_dense_page_ids.insert(current_dense_page_id);
	loop {
		let mut found_new = false;
		for linked_dense_page_id in page_graph.iter_forward_linked_pages(current_dense_page_id) {
			let linked_dense_page_id = linked_dense_page_id.to_native();
			if linked_dense_page_id == end_dense_page_id {
				return Some(visited_dense_page_id_trail.into_iter().collect());
			}
			if !found_new && !visited_dense_page_ids.contains(&linked_dense_page_id) {
				current_dense_page_id = linked_dense_page_id;
				found_new = true;
			}
		}
		if !found_new {
			match visited_dense_page_id_trail.pop_back() {
				Some(prev_dense_page_id) => {
					current_dense_page_id = prev_dense_page_id;
				}
				None => {
					println!("nothing more to look at");
					return None;
				}
			}
		}
		visited_dense_page_ids.insert(current_dense_page_id);
		visited_dense_page_id_trail.push_front(current_dense_page_id);
	}
}
