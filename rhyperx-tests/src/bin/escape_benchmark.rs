#![allow(dead_code)]

use std::error::Error;
use std::time::{Duration, Instant};

use rust_core::loader::DatasetLoader;
use rust_core::misc::{degeneracy_ordering, hyper_degeneracy_ordering};
use rust_core::motifs::algorithms::escape;
use rust_core::types::adj_list::{AdjList, Undirected, WithoutIncidence};
use rust_core::types::hyperadj_list::HyperAdjList;
use rust_core::types::{Hypergraph, NodeId, NodeWeight};

/// Computes the k-uniform density of a hypergraph: \rho = |E| / \binom{n}{k}
pub fn k_uniform_density(num_vertices: usize, num_edges: usize, k: usize) -> Option<f64> {
    if k == 0 || k > num_vertices {
        return None;
    }

    let max_possible_edges = binomial_coefficient(num_vertices, k)?;
    if max_possible_edges == 0 || num_edges > max_possible_edges {
        return None;
    }

    Some(num_edges as f64 / max_possible_edges as f64)
}

/// Safely computes \binom{n}{k} using u128 intermediate values to prevent overflow.
fn binomial_coefficient(n: usize, mut k: usize) -> Option<usize> {
    if k > n {
        return Some(0);
    }
    if k > n - k {
        k = n - k;
    }
    if k == 0 {
        return Some(1);
    }

    let mut res: u128 = 1;
    for i in 1..=k {
        res = res.checked_mul((n - k + i) as u128)?;
        res /= i as u128;
    }

    usize::try_from(res).ok()
}

#[derive(Debug, Clone, Default)]
struct BenchmarkResult {
    dataset_name: String,
    description: String,
    weighted: OrderResult,
    unweighted: OrderResult,
}

#[derive(Debug, Clone, Default)]
struct OrderResult {
    order3: ExecutionInfos,
    order4: ExecutionInfos,
}

#[derive(Debug, Clone, Default)]
struct ExecutionInfos {
    time: Duration,
    graph_infos: GraphInfos,
}

#[derive(Debug, Clone, Default)]
struct GraphInfos {
    n: usize,
    e2: usize,
    e3: usize,
    e4: usize,
    density2: f64,
    density3: f64,
    density4: f64,
    max_degree: usize,
    degeneracy: usize,
    hyper_degeneracy: usize,
}

fn get_graph_infos<W: Clone>(hg: &Hypergraph<NodeId, W>) -> GraphInfos {
    let n = hg.n();
    let count2 = hg.edges::<2>().len();
    let count3 = hg.edges::<3>().len();
    let count4 = hg.edges::<4>().len();

    let (hyper_adj, _, _) = HyperAdjList::<W>::from_hypergraph_mapped(hg.clone());
    let max_degree = hyper_adj
        .iter_all_incident_edges()
        .map(|neighbors| neighbors.len())
        .max()
        .unwrap_or(0);

    let (_, hyper_degeneracy) = hyper_degeneracy_ordering(&hyper_adj);

    let edges = hg
        .edges::<2>()
        .iter()
        .map(|e| (e.nodes[0], e.nodes[1], ()))
        .collect::<Vec<_>>();

    let (adj, _, _) = AdjList::<(), Undirected, WithoutIncidence>::from_edges_mapped(edges);
    let (_, degeneracy) = degeneracy_ordering(&adj);

    GraphInfos {
        n,
        e2: count2,
        e3: count3,
        e4: count4,
        density2: k_uniform_density(n, count2, 2).unwrap_or(0.0),
        density3: k_uniform_density(n, count3, 3).unwrap_or(0.0),
        density4: k_uniform_density(n, count4, 4).unwrap_or(0.0),
        max_degree,
        degeneracy,
        hyper_degeneracy,
    }
}

fn get_execution_time<F>(rounds: usize, f: F) -> Duration
where
    F: Fn(),
{
    let times = (0..rounds).map(|_| {
        let start = Instant::now();
        f();
        start.elapsed()
    });

    let min_time = times.clone().min().unwrap_or(Duration::ZERO);
    let _avg_time = times
        .clone()
        .reduce(|a, b| a + b)
        .map(|total| total / rounds as u32);
    min_time
}

const ROUNDS: usize = 5;

macro_rules! test_dataset {
    ($name: ident, $rv: expr) => {
        println!("Running benchmark for dataset: {}", stringify!($name));

        // --- 1. UNWEIGHTED PIPELINE ---
        let mut hg_unweighted = DatasetLoader::builder()
            .cached(true)
            .$name()
            .unweighted()
            .load()?
            .0;

        // Order 4
        hg_unweighted.retain_orders(&[2, 3, 4]);
        let (adj_order4, _, _) = HyperAdjList::<()>::from_hypergraph_mapped(hg_unweighted.clone());

        let unweighted_order4 = ExecutionInfos {
            time: get_execution_time(ROUNDS, || {
                escape::unweighted_4(&adj_order4);
            }),
            graph_infos: get_graph_infos(&hg_unweighted),
        };

        hg_unweighted.retain_orders(&[2, 3]);
        let (adj_order3, _, _) = HyperAdjList::<()>::from_hypergraph_mapped(hg_unweighted.clone());
        // Order 3
        let unweighted_order3 = ExecutionInfos {
            time: get_execution_time(ROUNDS, || {
                escape::unweighted_3(&adj_order3);
            }),
            graph_infos: get_graph_infos(&hg_unweighted),
        };

        // --- 2. WEIGHTED PIPELINE ---
        let mut hg_weighted = DatasetLoader::builder()
            .cached(true)
            .$name()
            .weighted()
            .load()?
            .0;

        // Order 4
        hg_weighted.retain_orders(&[2, 3, 4]);
        let (adj_weighted_4, _, _) =
            HyperAdjList::<NodeWeight>::from_hypergraph_mapped(hg_weighted.clone());
        let weighted_order4 = ExecutionInfos {
            time: get_execution_time(ROUNDS, || {
                escape::weighted_4(&adj_weighted_4);
            }),
            graph_infos: get_graph_infos(&hg_weighted),
        };

        // Order 3
        hg_weighted.retain_orders(&[2, 3]);
        let (adj_weighted_3, _, _) =
            HyperAdjList::<NodeWeight>::from_hypergraph_mapped(hg_weighted.clone());
        let weighted_order3 = ExecutionInfos {
            time: get_execution_time(3, || {
                escape::weighted_3(&adj_weighted_3);
            }),
            graph_infos: get_graph_infos(&hg_weighted),
        };

        $rv.push(BenchmarkResult {
            dataset_name: stringify!($name).to_string(),
            description: String::new(),
            unweighted: OrderResult {
                order3: unweighted_order3,
                order4: unweighted_order4,
            },
            weighted: OrderResult {
                order3: weighted_order3,
                order4: weighted_order4,
            },
        });
    };
}

fn format_duration_ms(d: Duration) -> String {
    if d.is_zero() {
        "{---}".to_string()
    } else {
        format!("{:.2}", d.as_secs_f64() * 1000.0)
    }
}

fn print_latex_timings(results: &[BenchmarkResult]) {
    println!("\n% --- LaTeX Table Output: Timings ---");
    println!("\\begin{{table}}[h]");
    println!("    \\centering");
    println!(
        "    \\begin{{tabular}}{{l S[table-format=3.2] S[table-format=3.2] S[table-format=4.2] c}}"
    );
    println!("        \\toprule");
    println!(
        "        & \\multicolumn{{2}}{{c}}{{Order 3 motifs}} & \\multicolumn{{2}}{{c}}{{Order 4 motifs}} \\\\"
    );
    println!("        \\cmidrule(lr){{2-3}} \\cmidrule(lr){{4-5}}");
    println!(
        "        Dataset & {{unweighted}} & {{weighted}} & {{unweighted}} & {{weighted}} \\\\"
    );
    println!("        \\midrule");

    for r in results {
        println!(
            "        \\verb|{:<18}| & {:<15} & {:<15} & {:<15} & {} \\\\",
            r.dataset_name,
            format_duration_ms(r.unweighted.order3.time),
            format_duration_ms(r.weighted.order3.time),
            format_duration_ms(r.unweighted.order4.time),
            format_duration_ms(r.weighted.order4.time)
        );
    }

    println!("        \\bottomrule");
    println!("    \\end{{tabular}}");
    println!("    \\caption{{Algorithm execution times by order, in milliseconds (ms).}}");
    println!("    \\label{{tab:benchmark_results}}");
    println!("\\end{{table}}");
}

fn print_latex_specs(results: &[BenchmarkResult]) {
    println!("\n% --- LaTeX Table Output: Dataset Structural Specifications ---");
    println!("\\begin{{table}}[h]");
    println!("    \\centering");
    println!(
        "    \\begin{{tabular}}{{l S[table-format=6.0] S[table-format=6.0] S[table-format=6.0] S[table-format=6.0] S[table-format=5.0] S[table-format=4.0] S[table-format=4.0]}}"
    );
    println!("        \\toprule");
    println!(
        "        Dataset & {{$n$}} & {{$e_2$}} & {{$e_3$}} & {{$e_4$}} & {{max deg}} & {{degen}} & {{hyper degen}} \\\\"
    );
    println!("        \\midrule");

    for r in results {
        let info = &r.unweighted.order4.graph_infos;
        println!(
            "        \\verb|{:<18}| & {:<8} & {:<8} & {:<8} & {:<8} & {:<8} & {:<8} & {} \\\\",
            r.dataset_name,
            info.n,
            info.e2,
            info.e3,
            info.e4,
            info.max_degree,
            info.degeneracy,
            info.hyper_degeneracy
        );
    }

    println!("        \\bottomrule");
    println!("    \\end{{tabular}}");
    println!("    \\caption{{Structural properties of the benchmarked datasets.}}");
    println!("    \\label{{tab:dataset_specs_structural}}");
    println!("\\end{{table}}");
}

fn print_latex_densities(results: &[BenchmarkResult]) {
    println!("\n% --- LaTeX Table Output: Dataset Densities ---");
    println!("\\begin{{table}}[h]");
    println!("    \\centering");
    println!(
        "    \\begin{{tabular}}{{l S[table-format=1.2e-2] S[table-format=1.2e-2] S[table-format=1.2e-2]}}"
    );
    println!("        \\toprule");
    println!(
        "        Dataset & {{density\\textsubscript{{2}}}} & {{density\\textsubscript{{3}}}} & {{density\\textsubscript{{4}}}} \\\\"
    );
    println!("        \\midrule");

    for r in results {
        let info = &r.unweighted.order4.graph_infos;
        println!(
            "        \\verb|{:<18}| & {:.3e} & {:.3e} & {:.3e} \\\\",
            r.dataset_name, info.density2, info.density3, info.density4
        );
    }

    println!("        \\bottomrule");
    println!("    \\end{{tabular}}");
    println!("    \\caption{{Density properties of the benchmarked datasets across orders.}}");
    println!("    \\label{{tab:dataset_specs_densities}}");
    println!("\\end{{table}}");
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut results = Vec::new();

    // Run benchmarks across datasets
    test_dataset!(hospital, results);
    test_dataset!(conference, results);
    test_dataset!(dblp, results);
    test_dataset!(enron, results);
    test_dataset!(eu, results);
    test_dataset!(geology, results);
    test_dataset!(high_school, results);
    test_dataset!(history, results);
    test_dataset!(justice, results);
    test_dataset!(ndc_classes, results);
    test_dataset!(ndc_substances, results);
    test_dataset!(pacs, results);
    test_dataset!(primary_school, results);
    test_dataset!(wiki, results);
    test_dataset!(workspace, results);
    test_dataset!(friendship_hs, results);

    print_latex_timings(&results);
    print_latex_specs(&results);
    print_latex_densities(&results);

    Ok(())
}
