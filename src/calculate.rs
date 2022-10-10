use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;
use std::iter::{once, ExactSizeIterator};
use std::marker::PhantomData;

use petgraph::algo::toposort;
use petgraph::graph::Graph;

use anyhow::{bail, Result};
use thiserror::Error;

// TODO: update number of errors and improve their descriptions
#[derive(Debug, Error)]
pub enum CalcError {
    #[error("invalid data")]
    Invalid,
    #[error("cycle detected in node {0:?}")]
    Cycle(String),
}

pub struct WGraph<'a, Item = &'a str, Index = u32>(Graph<Item, Index>, PhantomData<&'a ()>);

impl<'a, Item: Copy + Display, Index> WGraph<'a, Item, Index> {
    pub fn new(graph: Graph<Item, Index>) -> Self {
        WGraph(graph, PhantomData)
    }

    pub fn first_last(&self) -> Result<(Item, Item)> {
        let nodes = self.0.raw_nodes();
        let topo = toposort(&self.0, None).map_err(|x| {
            CalcError::Cycle(nodes.get(x.node_id().index()).unwrap().weight.to_string())
        })?;
        if topo.len() != nodes.len() {
            bail!("invalid length TODO")
        }
        let first = topo.first().ok_or(CalcError::Invalid)?;
        let first = nodes.get(first.index()).ok_or(CalcError::Invalid)?.weight;
        let last = topo.last().ok_or(CalcError::Invalid)?;
        let last = nodes.get(last.index()).ok_or(CalcError::Invalid)?.weight;

        Ok((first, last))
    }
}

pub trait AIter<'a>: ExactSizeIterator<Item = (&'a str, &'a str)> + Clone + 'a {}
impl<'a, T: ExactSizeIterator<Item = (&'a str, &'a str)> + Clone + 'a> AIter<'a> for T {}

pub trait AEdges<Iter>: IntoIterator<IntoIter = Iter> {}
impl<Iter, T: IntoIterator<IntoIter = Iter>> AEdges<Iter> for T {}

impl WGraph<'_> {
    pub fn from_edges<'b, Iter: AIter<'b>, Edges: AEdges<Iter>>(
        edges: Edges,
    ) -> Result<WGraph<'b>> {
        let edges = edges.into_iter();
        let len = edges.len();
        let nodes = Vec::from_iter(
            BTreeSet::from_iter(edges.clone().flat_map(|(s, e)| once(s).chain(once(e))))
                .into_iter(),
        );
        if nodes.len() > u32::MAX as usize {
            bail!("invalid data max length")
        }
        let hash_nodes: HashMap<&str, u32> = nodes
            .iter()
            .enumerate()
            .map(|(i, x)| (*x, i as u32))
            .collect();
        let mut graph: Graph<&str, u32> = Graph::with_capacity(nodes.len(), len);
        for node in &nodes {
            graph.add_node(*node);
        }
        for (start, end) in edges {
            graph.update_edge(hash_nodes[start].into(), hash_nodes[end].into(), 1);
        }

        Ok(WGraph::new(graph))
    }

    pub fn calc_first_last<'b, Iter: AIter<'b>, Edges: AEdges<Iter>>(
        edges: Edges,
    ) -> Result<(&'b str, &'b str)> {
        let graph = Self::from_edges(edges)?;
        graph.first_last()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert!(WGraph::calc_first_last([]).is_err());
        assert!(WGraph::calc_first_last([("foo", "foo")]).is_err());

        assert_eq!(
            WGraph::calc_first_last([("foo", "bar")]).unwrap(),
            ("foo", "bar")
        );

        let s = [("ATL", "EWR"), ("SFO", "ATL")];
        let result = WGraph::calc_first_last(s).unwrap();
        assert_eq!(result, ("SFO", "EWR"));

        let s = [
            ("IND", "EWR"),
            ("SFO", "ATL"),
            ("GSO", "IND"),
            ("ATL", "GSO"),
        ];
        let result = WGraph::calc_first_last(s).unwrap();
        assert_eq!(result, ("SFO", "EWR"));
    }
}
