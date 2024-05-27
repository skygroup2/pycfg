use std::collections::{BTreeMap, HashSet};
use fnv::FnvBuildHasher;
use revm::primitives::{Bytecode, Bytes};
use revm::interpreter::analysis::to_analysed;
use evm_cfg::cfg_gen::{cfg_graph, dasm, stack_solve};
use crate::block::{InstructionBlock};
use petgraph::Direction;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub enum Edges {
    Jump,           // Next instruction in sequence
    ConditionTrue,  // Conditional jumpi, true branch
    ConditionFalse, // Conditional jumpi, false branch
    SymbolicJump,   // Jump to a symbolic value
}

#[pyclass]
pub struct NodeEdge {
    #[pyo3(get)]
    pub from_node: (u16, u16),
    #[pyo3(get)]
    pub to_node: (u16, u16),
    #[pyo3(get)]
    pub edge: Edges
}

impl NodeEdge {
    pub fn new(from_node: (u16, u16), to_node: (u16, u16), edge: &cfg_graph::Edges) -> Self {
        Self{
            from_node,
            to_node,
            edge: Edges::from(edge)
        }
    }
}


#[pyclass]
pub struct CFGEvm {
    pub cfg_dag: cfg_graph::CFGDag,
    #[pyo3(get)]
    pub last_node: Option<(u16, u16)>,
    #[pyo3(get)]
    pub jumpi_edge: Option<Edges>,
    #[pyo3(get)]
    pub map_to_instructionblock: BTreeMap<(u16, u16), InstructionBlock>
}

#[pymethods]
impl CFGEvm {
    #[new]
    pub fn new(bytecode_string: String) -> Self {
        // sanitize bytecode string from newlines/spaces/etc
        let bytecode_string = bytecode_string.replace(['\n', ' ', '\r'], "");
        // get jumptable from revm
        let contract_data : Bytes = hex::decode(&bytecode_string ).unwrap().into();
        let bytecode_analysed = to_analysed(Bytecode::new_raw(contract_data));
        let revm_jumptable = bytecode_analysed.legacy_jump_table().expect("revm bytecode analysis failed");

        // convert jumptable to HashSet of valid jumpdests using as_slice
        let mut set_all_valid_jumpdests: HashSet<u16, FnvBuildHasher> = HashSet::default();
        let slice = revm_jumptable.as_slice();
        for (byte_index, &byte) in slice.iter().enumerate() {
            for bit_index in 0..8 {
                if byte & (1 << bit_index) != 0 {
                    let pc = (byte_index * 8 + bit_index) as u16;
                    set_all_valid_jumpdests.insert(pc);
                }
            }
        }

        // convert bytecode to instruction blocks
        let mut instruction_blocks = dasm::disassemble(bytecode_analysed.original_byte_slice().into());

        // analyze each instruction block statically to determine stack usage agnostic to entry values
        for block in &mut instruction_blocks {
            block.analyze_stack_info();
        }

        // QoL: map cfg-nodes to instruction blocks for easy lookup rather than stuffing graph with instruction block info as node weights
        let mut map_to_instructionblocks: BTreeMap<(u16, u16), dasm::InstructionBlock> = instruction_blocks
            .iter()
            .map(|block| ((block.start_pc, block.end_pc), block.clone()))
            .collect();

        // create initial cfg using only nodes
        let mut cfg_runner =
            cfg_graph::CFGRunner::new(bytecode_analysed.original_byte_slice().into(), &mut map_to_instructionblocks);

        // form basic edges based on direct pushes leading into jumps, false edges of jumpis, and pc+1 when no jump is used
        cfg_runner.form_basic_connections();
        // trim instruction blocks from graph that have no incoming edges and do not lead the block with a jumpdest
        cfg_runner.remove_unreachable_instruction_blocks();

        // find new edges based on indirect jumps
        let label_symbolic_jumps = false;
        stack_solve::symbolic_cycle(
            &mut cfg_runner,
            &set_all_valid_jumpdests,
            label_symbolic_jumps,
        );
        let mut jumpi_edge: Option<Edges> = None;
        if cfg_runner.jumpi_edge.is_some() {
            jumpi_edge = Some(Edges::from(&cfg_runner.jumpi_edge.unwrap()));
        }
        let mut map_to_instructionblock = BTreeMap::new();
        for (key, value) in cfg_runner.map_to_instructionblock.iter() {
            map_to_instructionblock.insert(key.clone(), InstructionBlock::from(value));
        }
        Self {
            cfg_dag: cfg_runner.cfg_dag.clone(),
            last_node: cfg_runner.last_node.clone(),
            jumpi_edge,
            map_to_instructionblock
        }
    }

    pub fn incoming_edges(&self, node: (u16, u16)) -> Vec<NodeEdge> {
        let in_edges = self.cfg_dag.edges_directed(node, Direction::Incoming);
        let mut ret : Vec<NodeEdge> = vec![];
        for e in in_edges.into_iter() {
            ret.push(NodeEdge::new(e.0, e.1, e.2));
        }
        return ret
    }

    pub fn outgoing_edges(&self, node: (u16, u16)) -> Vec<NodeEdge> {
        let in_edges = self.cfg_dag.edges_directed(node, Direction::Outgoing);
        let mut ret : Vec<NodeEdge> = vec![];
        for e in in_edges.into_iter() {
            ret.push(NodeEdge::new(e.0, e.1, e.2));
        }
        return ret
    }
}


impl From<&cfg_graph::Edges> for Edges {
    fn from(value: &cfg_graph::Edges) -> Self {
        match value {
            cfg_graph::Edges::Jump => {
                Self::Jump
            }
            cfg_graph::Edges::ConditionTrue => {
                Self::ConditionTrue
            }
            cfg_graph::Edges::ConditionFalse => {
                Self::ConditionFalse
            }
            cfg_graph::Edges::SymbolicJump => {
                Self::SymbolicJump
            }
        }
    }
}