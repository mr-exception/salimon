//! Immutable epoch snapshot. Conservative sphere bounds prune whole source
//! groups without guessing gravity from distance alone or omitting large masses.
use super::*;
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct QueryWork {
    pub visited_nodes: usize,
    pub evaluated_sources: usize,
}
struct Node {
    center: GalacticPosition,
    radius: f64,
    max_mu: f64,
    range: std::ops::Range<usize>,
    children: Option<(usize, usize)>,
}
pub struct SourceIndex {
    sources: Vec<Source>,
    ids: BTreeMap<CatalogId, usize>,
    nodes: Vec<Node>,
}
#[derive(Clone, Copy)]
struct Search<'a> {
    position: GalacticPosition,
    count: usize,
    old: &'a [CatalogId],
    hysteresis: f64,
}
impl SourceIndex {
    pub fn new(mut sources: Vec<Source>) -> Result<Self, Error> {
        if sources.len() > 1_000_000 {
            return Err(invalid("gravity spatial index source bound"));
        }
        let mut ids = BTreeSet::new();
        for s in &sources {
            if !ids.insert(s.id.clone())
                || !s.mu_m3_per_s2.is_finite()
                || s.mu_m3_per_s2 <= 0.0
                || !s.radius_meters.is_finite()
                || s.radius_meters < 0.0
            {
                return Err(invalid("gravity index duplicate/physical properties"));
            }
        }
        let mut nodes = Vec::new();
        let len = sources.len();
        if len > 0 {
            Self::build(&mut sources, 0..len, &mut nodes)?;
        }
        let ids = sources
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();
        Ok(Self {
            sources,
            ids,
            nodes,
        })
    }
    fn build(
        sources: &mut [Source],
        range: std::ops::Range<usize>,
        nodes: &mut Vec<Node>,
    ) -> Result<usize, Error> {
        let center = sources[range.start].position;
        let mut radius = 0.0_f64;
        let mut max_mu = 0.0_f64;
        let mut spans = [0.0_f64; 3];
        for s in &sources[range.clone()] {
            let d = s.position.relative_meters(center)?;
            radius = radius.max(d[0].hypot(d[1]).hypot(d[2]) + s.radius_meters);
            max_mu = max_mu.max(s.mu_m3_per_s2);
            for i in 0..3 {
                spans[i] = spans[i].max(d[i].abs());
            }
        }
        // Expand, never shrink, around f64 distance/relative-subtraction error.
        let radius = radius * (1.0 + 1e-12) + 1e-6;
        let n = nodes.len();
        nodes.push(Node {
            center,
            radius,
            max_mu,
            range: range.clone(),
            children: None,
        });
        if range.len() > 8 {
            let axis = (0..3)
                .max_by(|a, b| spans[*a].total_cmp(&spans[*b]))
                .expect("three axes");
            sources[range.clone()].sort_by(|a, b| {
                a.position.cells()[axis]
                    .cmp(&b.position.cells()[axis])
                    .then_with(|| {
                        a.position.offset_meters()[axis]
                            .total_cmp(&b.position.offset_meters()[axis])
                    })
                    .then_with(|| a.id.cmp(&b.id))
            });
            let mid = range.start + range.len() / 2;
            let left = Self::build(sources, range.start..mid, nodes)?;
            let right = Self::build(sources, mid..range.end, nodes)?;
            nodes[n].children = Some((left, right));
        }
        Ok(n)
    }
    fn upper(&self, node: usize, position: GalacticPosition) -> Result<f64, Error> {
        let n = &self.nodes[node];
        let d = n.center.relative_meters(position)?;
        let distance = d[0].hypot(d[1]).hypot(d[2]);
        let lower = (distance * (1.0 - 1e-12) - n.radius).max(0.0);
        Ok(if lower == 0.0 {
            f64::INFINITY
        } else {
            n.max_mu / lower / lower
        })
    }
    fn visit(
        &self,
        node: usize,
        query: &Search<'_>,
        best: &mut Vec<(f64, usize)>,
        work: &mut QueryWork,
    ) -> Result<(), Error> {
        let Search {
            position,
            count,
            old,
            hysteresis,
        } = *query;
        work.visited_nodes += 1;
        if best.len() == count
            && self.upper(node, position)? < best.last().expect("positive full count").0
        {
            return Ok(());
        }
        let n = &self.nodes[node];
        if let Some((a, b)) = n.children {
            let (first, last) = if self.upper(a, position)? >= self.upper(b, position)? {
                (a, b)
            } else {
                (b, a)
            };
            self.visit(first, query, best, work)?;
            self.visit(last, query, best, work)?;
        } else {
            for i in n.range.clone() {
                let s = &self.sources[i];
                work.evaluated_sources += 1;
                let d = s.position.relative_meters(position)?;
                let r = d[0].hypot(d[1]).hypot(d[2]);
                if r <= s.radius_meters || r == 0.0 {
                    return Err(invalid(
                        "gravity indexed source interior; contact adapter required",
                    ));
                }
                let score = s.mu_m3_per_s2
                    / r
                    / r
                    / (if old.contains(&s.id) {
                        1.0
                    } else {
                        1.0 + hysteresis
                    });
                best.push((score, i));
                best.sort_by(|(sa, a), (sb, b)| {
                    sb.total_cmp(sa)
                        .then_with(|| self.sources[*a].id.cmp(&self.sources[*b].id))
                });
                best.truncate(count);
            }
        }
        Ok(())
    }
    pub fn candidates(
        &self,
        position: GalacticPosition,
        count: usize,
        old: &[CatalogId],
        retained: &[CatalogId],
        hysteresis: f64,
    ) -> Result<(Vec<Source>, QueryWork), Error> {
        if count == 0
            || count > 64
            || !hysteresis.is_finite()
            || !(0.0..=0.5).contains(&hysteresis)
            || old.len() > 128
            || retained.len() > 128
        {
            return Err(invalid("gravity indexed query configuration"));
        }
        let query = Search {
            position,
            count,
            old,
            hysteresis,
        };
        let mut best = Vec::new();
        let mut work = QueryWork::default();
        if !self.nodes.is_empty() {
            self.visit(0, &query, &mut best, &mut work)?;
        }
        let mut result: BTreeMap<CatalogId, Source> = best
            .into_iter()
            .map(|(_, i)| (self.sources[i].id.clone(), self.sources[i].clone()))
            .collect();
        for id in retained {
            let i = self
                .ids
                .get(id)
                .ok_or_else(|| invalid("active gravity source missing from index snapshot"))?;
            result.insert(id.clone(), self.sources[*i].clone());
        }
        Ok((result.into_values().collect(), work))
    }
}
impl Selector {
    pub fn sample_indexed(
        &mut self,
        index: &SourceIndex,
        position: GalacticPosition,
        dt_seconds: f64,
    ) -> Result<(Acceleration, QueryWork), Error> {
        let retained: Vec<_> = self
            .transition
            .as_ref()
            .map(|t| t.from.iter().chain(&t.to).cloned().collect())
            .unwrap_or_else(|| self.selected.clone());
        let (sources, work) = index.candidates(
            position,
            self.config.contributors,
            &self.selected,
            &retained,
            self.config.hysteresis_fraction,
        )?;
        Ok((self.sample(&sources, position, dt_seconds)?, work))
    }
}
