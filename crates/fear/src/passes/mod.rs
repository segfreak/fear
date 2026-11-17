use std::collections::HashMap;

use crate::*;
use target_lexicon::Triple;

use super::{FuncId, Module};

pub mod cfg_simplify;
pub mod constfold;
pub mod copyprop;
pub mod cse;
pub mod dce;
pub mod dse;
pub mod gvn;
pub mod mem2reg;
pub mod simplify;
pub mod strength_reduction;
pub mod tcf;
pub mod tre;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    ConstantFolding,
    CopyPropagation,
    DeadCodeElimination,
    StrengthReduction,
    CommonSubexpressionElimination,
    DeadStoreElimination,
    TailRecursionElimination,
    Simplify,
    GlobalValueNumbering,
    CFGSimplify,
    Mem2Reg,
    TrivialComparesFolding,
    SCCP,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptConfig {
    // optimization level
    pub level: OptLevel,
}

impl PassManager {
    pub fn run_function(cfg: OptConfig, triple: &Triple, m: &mut Module, f: FuncId) -> PassResult {
        log::trace!("running optimizer via: {:?}", cfg);
        let mut result = PassResult::default();

        macro_rules! fnhash {
            () => {
                m.get_function_mut(f)
                    .unwrap()
                    .get_definition()
                    .unwrap()
                    .get_hash()
            };
        }

        let before_hash = fnhash!();

        loop {
            let mut run = Vec::new();
            let before_hash = fnhash!();

            macro_rules! run_pass {
                ($pass:expr, $kind:expr) => {
                    if $pass(m, f, triple) {
                        log::trace!("performed pass: {:?}", $kind);
                        run.push($kind);
                        m.get_function_mut(f)
                            .unwrap()
                            .get_definition_mut()
                            .unwrap()
                            .reconstruct();
                    }
                };
            }

            if cfg.level >= OptLevel::Default {
                run_pass!(constfold::constfold, PassKind::ConstantFolding);
                run_pass!(simplify::simplify, PassKind::Simplify);
                run_pass!(
                    strength_reduction::strength_reduction,
                    PassKind::StrengthReduction
                );
                run_pass!(copyprop::copyprop, PassKind::CopyPropagation);
                run_pass!(gvn::gvn, PassKind::GlobalValueNumbering);
                run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
                run_pass!(cfg_simplify::cfg_simplify, PassKind::CFGSimplify);
                run_pass!(tcf::tcf, PassKind::TrivialComparesFolding);
                run_pass!(dse::dse, PassKind::DeadStoreElimination);
                run_pass!(dce::dce, PassKind::DeadCodeElimination);
                run_pass!(tre::tre, PassKind::TailRecursionElimination);
                run_pass!(mem2reg::mem2reg, PassKind::Mem2Reg);
            }

            let changed = fnhash!() != before_hash;
            if changed {
                log::debug!("pipeline: performed {:?} passes", run);
            }

            if !changed {
                break;
            }

            result.passes.extend(run);
        }

        result.changed = fnhash!() != before_hash;

        result
    }

    pub fn run_module(
        cfg: OptConfig,
        triple: &Triple,
        m: &mut Module,
    ) -> HashMap<FuncId, PassResult> {
        let mut tmp = HashMap::new();

        let func_ids: Vec<FuncId> = m
            .iter_functions()
            .filter_map(|(id, f)| {
                if f.get_definition().is_some() {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        for func_id in func_ids {
            let pass_result = PassManager::run_function(cfg, triple, m, func_id);

            let func_name = &m.functions[&func_id].name;

            log::debug!(
                "optimiser has performed {} passes for the function {}: {:?}",
                pass_result.passes.len(),
                func_name,
                pass_result.passes
            );

            if pass_result.changed {
                log::trace!("after passes:\n{}", m.dump_function(func_id));
            }

            tmp.insert(func_id, pass_result);
        }

        tmp
    }
}

impl Module {
    pub fn optimize_with_options(
        &mut self,
        cfg: OptConfig,
        triple: &Triple,
    ) -> HashMap<FuncId, PassResult> {
        PassManager::run_module(cfg, triple, self)
    }

    pub fn optimize(&mut self, level: OptLevel, triple: &Triple) -> HashMap<FuncId, PassResult> {
        let cfg = OptConfig { level };
        self.optimize_with_options(cfg, triple)
    }
}
