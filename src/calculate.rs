use std::collections::{BTreeSet, HashMap};
use std::iter::once;

use petgraph::algo::toposort;
use petgraph::graph::Graph;

use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Edges(pub Vec<(String, String)>);

#[derive(Debug, Error)]
pub enum CalcError {
    #[error("invalid data")]
    Invalid,
    #[error("cycle detected in node {0:?}")]
    Cycle(String),
}

pub fn calc<'a, E: AsRef<[(&'a str, &'a str)]>>(edges: E) -> Result<(&'a str, &'a str), CalcError> {
    let s = edges.as_ref();
    let nodes: Vec<&str> = BTreeSet::from_iter(s.iter().flat_map(|(s, e)| once(s).chain(once(e))))
        .into_iter()
        .copied()
        .collect();
    if nodes.len() > u32::MAX as usize {
        return Err(CalcError::Invalid);
    }
    let hash_nodes: HashMap<&str, u32> = nodes
        .iter()
        .enumerate()
        .map(|(i, x)| (*x, i as u32))
        .collect();
    let mut graph: Graph<&str, u32> = Graph::with_capacity(nodes.len(), s.len());
    for node in &nodes {
        graph.add_node(*node);
    }
    for (start, end) in s {
        graph.update_edge(
            hash_nodes
                .get(start)
                .copied()
                .expect("should be imposible")
                .into(),
            hash_nodes
                .get(end)
                .copied()
                .expect("should be imposible")
                .into(),
            1,
        );
    }

    let mut topo = toposort(&graph, None)
        .map_err(|x| CalcError::Cycle(nodes.get(x.node_id().index()).unwrap().to_string()))?
        .into_iter();
    if topo.len() != nodes.len() {
        return Err(CalcError::Invalid);
    }
    let first = topo.next().ok_or(CalcError::Invalid)?;
    let first = nodes.get(first.index()).ok_or(CalcError::Invalid)?;
    let last = topo.last().ok_or(CalcError::Invalid)?;
    let last = nodes.get(last.index()).ok_or(CalcError::Invalid)?;

    Ok((first, last))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert!(calc([]).is_err());
        assert!(calc([("foo", "foo")]).is_err());

        assert_eq!(calc([("foo", "bar")]).unwrap(), ("foo", "bar"));

        let s = [("ATL", "EWR"), ("SFO", "ATL")];
        let result = calc(s).unwrap();
        assert_eq!(result, ("SFO", "EWR"));

        let s = [
            ("IND", "EWR"),
            ("SFO", "ATL"),
            ("GSO", "IND"),
            ("ATL", "GSO"),
        ];
        let result = calc(s).unwrap();
        assert_eq!(result, ("SFO", "EWR"));
    }
}
