use crate::{ArchivedPageGraph, DensePageId};

pub fn speedrun_shortest_path(
	start_dense_page_id: DensePageId,
	end_dense_page_id: DensePageId,
	page_graph: &ArchivedPageGraph,
) -> Option<Vec<DensePageId>> {
	// same as speedrun_any_percent_bidirectional_bfs but it does not exit on success but tries
	// further
	todo!()
}
