//! Flow index for data flow tracking

use hodei_ir::{Fact, FactId, FactType, FlowId};
use std::collections::HashMap;

/// Simple flow index (placeholder for petgraph)
#[derive(Debug, Clone)]
pub struct FlowIndex {
    flows: HashMap<FlowId, Vec<FactId>>, // flow_id -> list of facts in this flow
}

impl FlowIndex {
    pub fn build(facts: &[&Fact]) -> Self {
        let mut flows = HashMap::new();

        for fact in facts {
            // Extract flow_id from TaintSource facts (simplified)
            if let FactType::TaintSource { flow_id, .. } = &fact.fact_type {
                flows.entry(*flow_id).or_insert_with(Vec::new).push(fact.id);
            }
        }

        Self { flows }
    }

    pub fn get_facts_for_flow(&self, flow_id: &FlowId) -> Option<&[FactId]> {
        self.flows.get(flow_id).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::FlowId;

    #[test]
    fn test_flow_index() {
        let facts: Vec<&Fact> = vec![];
        let index = FlowIndex::build(&facts);
        let test_flow_id = FlowId::new_uuid();
        assert!(index.get_facts_for_flow(&test_flow_id).is_none());
    }
}
