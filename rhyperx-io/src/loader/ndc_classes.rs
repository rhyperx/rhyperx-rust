use rhyperx_core::hypergraph::{HxUnsizedRef, Hypergraph};

use crate::loader::common::{Loader, build_edges_from_nverts_simplices, read_nverts_simplices};
use crate::loader::error::LoaderError;

use super::{NdcClassesStdUnweightedLoader, NdcClassesStdWeightedLoader};

impl Loader for NdcClassesStdUnweightedLoader {
    type Output = Hypergraph<u32, ()>;

    const VARIANT: &'static str = "uw";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let (v, s) = read_nverts_simplices(&dataset_location)?;

        let mut hg = Hypergraph::new();

        for mut e in build_edges_from_nverts_simplices(v, &s) {
            hg.add_edge_slice(&mut e, ()).expect("Malformed edge");
        }

        Ok(hg)
    }
}

impl Loader for NdcClassesStdWeightedLoader {
    type Output = Hypergraph<u32, f32>;

    const VARIANT: &'static str = "w";

    fn from_file(&self) -> Result<Self::Output, LoaderError> {
        let dataset_location = self.dataset_location.clone();
        let (v, s) = read_nverts_simplices(&dataset_location)?;

        let mut hg = Hypergraph::new();

        for mut e in build_edges_from_nverts_simplices(v, &s) {
            e.sort_unstable();
            match hg.get_hyperedge_mut(HxUnsizedRef::new(&e, &0.0)) {
                Some(r) => *r.weight += 1.0,
                None => {
                    hg.add_edge_slice(&mut e, 1.0).expect("Malformed edge");
                }
            }
        }

        Ok(hg)
    }
}
