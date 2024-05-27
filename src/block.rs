use std::collections::{BTreeSet, HashMap, HashSet};
use fnv::FnvBuildHasher;
use evm_cfg::cfg_gen::{dasm};
use pyo3::pyclass;

#[pyclass]
#[derive(Clone, Default)]
pub struct InstructionBlock {
    #[pyo3(get)]
    pub start_pc: u16,
    #[pyo3(get)]
    pub end_pc: u16,
    #[pyo3(get)]
    pub ops: Vec<(u16, u8, Option<Vec<u8>>)>, // Vec<pc, op_code, push_val
    #[pyo3(get)]
    pub indirect_jump: Option<u16>,
    #[pyo3(get)]
    pub push_vals: Vec<(Vec<u8>, Option<BTreeSet<u16>>)>,
    #[pyo3(get)]
    pub stack_info: StackInfo,
}

#[pyclass]
#[derive(Clone, Default, Eq, PartialEq)]
pub struct StackInfo {
    #[pyo3(get)]
    pub min_stack_size_required_for_entry: u16, // essentially the largest key within map_stack_entry_pos_to_stack_usage_pos
    #[pyo3(get)]
    pub stack_entry_pos_to_op_usage:
    HashMap<u16, HashSet<(u16, dasm::OpWithPos), FnvBuildHasher>, FnvBuildHasher>, // stack_pos: (pc, OpWithPos)
    #[pyo3(get)]
    pub stack_entry_pos_to_stack_exit_pos:
    HashMap<u16, HashSet<u16, FnvBuildHasher>, FnvBuildHasher>, // {entry_pos: [exit_pos1, exit_pos2, ...]}
    #[pyo3(get)]
    pub stack_size_delta: i16, // how much the stack size changes from entry to exit
    #[pyo3(get)]
    pub push_used_for_jump: Option<u16>, // if a push is used for a jump, this is the value of the push
}

impl From<&dasm::StackInfo> for StackInfo {
    fn from(value: &dasm::StackInfo) -> Self {
        Self{
            min_stack_size_required_for_entry: value.min_stack_size_required_for_entry.clone(),
            stack_entry_pos_to_op_usage: value.stack_entry_pos_to_op_usage.clone(),
            stack_entry_pos_to_stack_exit_pos: value.stack_entry_pos_to_stack_exit_pos.clone(),
            stack_size_delta: value.stack_size_delta.clone(),
            push_used_for_jump: value.push_used_for_jump.clone()
        }
    }
}

impl From<&dasm::InstructionBlock> for InstructionBlock {
    fn from(value: &dasm::InstructionBlock) -> Self {
        Self{
            start_pc: value.start_pc.clone(),
            end_pc: value.end_pc.clone(),
            ops: value.ops.clone(),
            indirect_jump: value.indirect_jump.clone(),
            push_vals: value.push_vals.clone(),
            stack_info: StackInfo::from(&value.stack_info)
        }
    }
}