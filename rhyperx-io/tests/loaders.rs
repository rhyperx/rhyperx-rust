// use std::io::Write;
// use std::path::PathBuf;
//
// use rhyperx_core::hypergraph::Hypergraph;
//
// use rhyperx_io::loader::{ConferenceStdUnweightedLoader, ConferenceStdWeightedLoader};
// use rhyperx_io::loader::{DblpStdUnweightedLoader, DblpStdWeightedLoader};
// use rhyperx_io::loader::{EuStdUnweightedLoader, EuStdWeightedLoader};
//
// fn write_temp(contents: &str) -> PathBuf {
//     let mut file = std::env::temp_dir();
//     file.push(format!(
//         "rhyperx_io_test_{}_{}",
//         std::process::id(),
//         rand_suffix()
//     ));
//     let mut f = std::fs::File::create(&file).unwrap();
//     f.write_all(contents.as_bytes()).unwrap();
//     file
// }
//
// fn rand_suffix() -> u64 {
//     use std::time::{SystemTime, UNIX_EPOCH};
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_nanos() as u64
// }
//
// fn cleanup(path: &PathBuf) {
//     let _ = std::fs::remove_file(path);
//     let _ = std::fs::remove_file(path.with_extension("bin"));
// }
//
// fn edge_sets(hg: &Hypergraph<u32, ()>) -> Vec<Vec<u32>> {
//     let mut sets: Vec<Vec<u32>> = hg
//         .iter_hg_sizes()
//         .flat_map(|size| hg.iter_edges(size))
//         .map(|e| e.nodes.to_vec())
//         .collect();
//     for s in &mut sets {
//         s.sort_unstable();
//     }
//     sets.sort_unstable();
//     sets
// }
//
// fn edge_counts(hg: &Hypergraph<u32, f32>) -> Vec<(Vec<u32>, f32)> {
//     let mut v: Vec<(Vec<u32>, f32)> = hg
//         .iter_hg_sizes()
//         .flat_map(|size| hg.iter_edges(size))
//         .map(|e| {
//             let mut nodes = e.nodes.to_vec();
//             nodes.sort_unstable();
//             (nodes, *e.weight)
//         })
//         .collect();
//     v.sort_by(|a, b| a.0.cmp(&b.0));
//     v
// }
//
// #[test]
// fn dblp_unweighted() {
//     let path = write_temp("paper,author\np1,1\np1,2\np1,3\np2,4\np2,5\n");
//     let mut loader = DblpStdUnweightedLoader::default();
//     loader.dataset_location = path.clone();
//     loader.cache_dir = None;
//
//     let hg = loader.load().unwrap();
//     let sets = edge_sets(&hg);
//     assert_eq!(sets, vec![vec![1, 2, 3], vec![4, 5]]);
//
//     cleanup(&path);
// }
//
// #[test]
// fn dblp_weighted_deduplicates() {
//     let path = write_temp("p1,1\np1,1\np1,2\np1,3\np1,3\n");
//     let mut loader = DblpStdWeightedLoader::default();
//     loader.dataset_location = path.clone();
//     loader.cache_dir = None;
//
//     let hg = loader.load().unwrap();
//     // {1,2,3} appears once (author ids deduped per paper); weight 1.0
//     assert_eq!(edge_counts(&hg), vec![(vec![1, 2, 3], 1.0)]);
//
//     cleanup(&path);
// }
//
// #[test]
// fn eu_unweighted_nverts_simplices() {
//     // one 3-edge then one 2-edge
//     let path = write_temp("");
//     let _ = std::fs::remove_file(&path);
//     std::fs::create_dir_all(&path).unwrap();
//     let name = path.file_name().unwrap().to_str().unwrap();
//     std::fs::write(path.join(format!("{name}-nverts.txt")), "3\n2\n").unwrap();
//     std::fs::write(
//         path.join(format!("{name}-simplices.txt")),
//         "10\n20\n30\n7\n8\n",
//     )
//     .unwrap();
//
//     let mut loader = EuStdUnweightedLoader::default();
//     loader.dataset_location = path.clone();
//     loader.cache_dir = None;
//
//     let hg = loader.load().unwrap();
//     assert_eq!(edge_sets(&hg), vec![vec![7, 8], vec![10, 20, 30]]);
//
//     let _ = std::fs::remove_dir_all(path);
// }
//
// #[test]
// fn eu_weighted_counts_identical_edges() {
//     let path = write_temp("");
//     let _ = std::fs::remove_file(&path);
//     std::fs::create_dir_all(&path).unwrap();
//     let name = path.file_name().unwrap().to_str().unwrap();
//     // two identical 2-edges {7,8}
//     std::fs::write(path.join(format!("{name}-nverts.txt")), "2\n2\n").unwrap();
//     std::fs::write(path.join(format!("{name}-simplices.txt")), "7\n8\n7\n8\n").unwrap();
//
//     let mut loader = EuStdWeightedLoader::default();
//     loader.dataset_location = path.clone();
//     loader.cache_dir = None;
//
//     let hg = loader.load().unwrap();
//     assert_eq!(edge_counts(&hg), vec![(vec![7, 8], 2.0)]);
//
//     let _ = std::fs::remove_dir_all(path);
// }
//
// #[test]
// fn conference_unweighted_cliques() {
//     // two time steps: a triangle (1,2,3) and a single edge (4,5)
//     let path = write_temp("32521 1 2\n32521 2 3\n32521 1 3\n32522 4 5\n");
//     let mut loader = ConferenceStdUnweightedLoader::default();
//     loader.dataset_location = path.clone();
//     loader.cache_dir = None;
//
//     let hg = loader.load().unwrap();
//     assert_eq!(edge_sets(&hg), vec![vec![1, 2, 3], vec![4, 5]]);
//
//     cleanup(&path);
// }
//
// #[test]
// fn conference_weighted_counts_cliques() {
//     // same triangle appears in two distinct time steps -> weight 2
//     let path = write_temp("32521 1 2\n32521 2 3\n32521 1 3\n32522 1 2\n32522 2 3\n32522 1 3\n");
//     let mut loader = ConferenceStdWeightedLoader::default();
//     loader.dataset_location = path.clone();
//     loader.cache_dir = None;
//
//     let hg = loader.load().unwrap();
//     assert_eq!(edge_counts(&hg), vec![(vec![1, 2, 3], 2.0)]);
//
//     cleanup(&path);
// }
