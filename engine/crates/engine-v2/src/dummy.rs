#![allow(unused)]

use std::num::NonZero;

use engine::Positioned;
use engine_parser::types::*;
use itertools::Itertools;
use tracing::instrument;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct ResolverDecisionId(NonZero<u16>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct EdgeId(NonZero<u16>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct SubgraphId(NonZero<u16>);

#[derive(id_derives::IndexedFields)]
struct PlanningTree {
    #[indexed_by(ResolverDecisionId)]
    decisions: Vec<ResolverDecision>,
    root: ResolverDecisionTree,
}

struct ResolverDecisionTree {
    fields: Vec<ResolverDecision>,
}

struct ResolverDecision {
    possibilities: Vec<(SubgraphId, ResolverDecisionTree)>,
}

#[derive(id_derives::IndexedFields)]
struct Context {
    #[indexed_by(SubgraphId)]
    subgraphs: Vec<Vec<&'static str>>,
    #[indexed_by(EdgeId)]
    edges: Vec<(String, String)>,
    edges_weight: Vec<u16>,
}

impl Context {
    fn plan(&self, query: &str) -> (usize, Vec<(SubgraphId, String)>) {
        let query = engine_parser::parse_query(query).unwrap();
        let DocumentOperations::Single(Positioned { node: op, .. }) = &query.operations else {
            unreachable!()
        };

        tracing::info_span!("root").in_scope(|| self.plan_selection_set(0usize.into(), &op.selection_set.node))
    }

    fn requires(&self, subgraph_id: usize, field: &str) -> Option<Vec<(usize, &'static str)>> {
        match (subgraph_id, field) {
            (3, "book") => Some(vec![(1, "a")]),
            (4, "book") => Some(vec![(1, "b")]),
            (1, "a") => Some(vec![(2, "c")]),
            _ => None,
        }
    }

    fn plan_selection_set(
        &self,
        parent_subgraph_id: SubgraphId,
        selection_set: &SelectionSet,
    ) -> (usize, Vec<(SubgraphId, String)>) {
        let mut obvious = Vec::new();
        let mut multiple = Vec::new();
        let mut edges = Vec::new();

        for (field_id, Positioned { node: selection, .. }) in selection_set.items.iter().enumerate() {
            let Selection::Field(Positioned { node: field, .. }) = selection else {
                unreachable!()
            };

            let field = selection.as_field().unwrap().name.node.as_str();
            let mut subgraphs = self
                .subgraphs
                .iter()
                .positions(|subgraph| subgraph.contains(&field))
                .map(SubgraphId::from);
            let first = subgraphs.next().expect("not plannable");
            if let Some(second) = subgraphs.next() {
                let mut candidates = vec![(first, field_id), (second, field_id)];
                candidates.extend(subgraphs.map(|i| (i, field_id)));

                multiple.push(candidates);
            } else {
                obvious.push((first, field_id));
            };
        }

        let mut min_cost = usize::MAX;
        let mut best_edges = Vec::new();
        'cases: for mut case in multiple.into_iter().multi_cartesian_product() {
            case.extend(obvious.iter().cloned());
            case.sort_unstable();
            let mut cost = case
                .iter()
                .filter_map(|(subgraph_id, _)| {
                    if *subgraph_id != parent_subgraph_id {
                        Some(subgraph_id)
                    } else {
                        None
                    }
                })
                .dedup()
                .count();
            let mut case_edges = Vec::new();
            if cost >= min_cost {
                continue;
            }
            for (subgraph_id, field_id) in case {
                let field = selection_set.items[field_id].as_field().unwrap();
                case_edges.push((subgraph_id, field.name.node.to_string()));
                if field.selection_set.items.is_empty() {
                    continue;
                }
                let (subselection_cost, subselection_edges) =
                    tracing::info_span!("subselection", field = field.name.as_str())
                        .in_scope(|| self.plan_selection_set(subgraph_id, &field.selection_set));
                cost += subselection_cost;
                if cost >= min_cost {
                    continue 'cases;
                }

                case_edges.extend(subselection_edges);
            }
            tracing::info!("Cost is {cost}");
            min_cost = cost;
            best_edges = case_edges;
        }

        edges.extend(best_edges);

        (min_cost, edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dummy() {
        let filter = tracing_subscriber::filter::EnvFilter::builder()
            .parse(std::env::var("RUST_LOG").unwrap_or("engine_v2=debug".to_string()))
            .unwrap();

        tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(filter)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .without_time()
            .init();

        let subgraphs = vec![
            Vec::new(),
            vec!["a", "b"],
            vec!["c"],
            vec!["book", "author", "title"],
            vec!["book", "cook", "knive"],
            vec!["cook", "kitchen"],
        ];
        let ctx = Context { subgraphs };

        println!("{:#?}", ctx.plan("{ author { cook { kitchen } book { title } } }"));

        unreachable!();
    }
}
