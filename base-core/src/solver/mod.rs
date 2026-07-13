/// Constraint Solver — otimização para seleção de componentes.
///
/// Suporta três modos:
/// - ILP (good_lp): otimização de custo via programação linear inteira mista
/// - Z3 (SMT): constraints lógicas (requer feature solver_z3)
/// - Heurístico: fallback sempre disponível
use crate::component_db::{ComponentCategory, ComponentDb, ComponentEntry};
use crate::spec::types::{BlockKind, DmaRequirement, FunctionalBlock, SystemConstraints};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Solution {
    pub assignments: HashMap<String, String>,
    pub scores: HashMap<String, f64>,
    pub total_cost: f64,
    pub feasible: bool,
}

/// Resolve usando ILP (good_lp) se disponível, senão usa heurística
pub fn solve(db: &ComponentDb, blocks: &[FunctionalBlock], budget: f64) -> Solution {
    #[cfg(feature = "solver_ilp")]
    {
        match ilp_optimize(db, blocks, budget) {
            Ok(sol) => {
                tracing::info!("[Solver] ILP solution: ${:.2}, {} components", sol.total_cost, sol.assignments.len());
                return sol;
            }
            Err(e) => tracing::warn!("[Solver] ILP failed: {}", e),
        }
    }

    #[cfg(feature = "solver_z3")]
    {
        match z3_solve(db, blocks) {
            Ok(sol) if sol.feasible => {
                tracing::info!("[Solver] Z3 solution: {} components", sol.assignments.len());
                return sol;
            }
            Ok(_) => tracing::warn!("[Solver] Z3 unsatisfiable"),
            Err(e) => tracing::warn!("[Solver] Z3 error: {}", e),
        }
    }

    heuristic_solve(db, blocks)
}

/// Solver ILP (requer feature solver_ilp)
#[cfg(feature = "solver_ilp")]
pub fn ilp_optimize(db: &ComponentDb, blocks: &[FunctionalBlock], budget: f64) -> Result<Solution, String> {
    use good_lp::{variable, ProblemVariables, SolverModel, Expression, Constraint, default_solver};

    let total: usize = blocks.iter()
        .map(|b| get_candidates(db, &b.kind).len())
        .sum();

    if total == 0 {
        return Err("No candidates found".into());
    }

    let mut vars = ProblemVariables::new();
    let mut x = Vec::new();
    for i in 0..total {
        x.push(vars.add(variable().min(0).max(1).name(format!("x{}", i))));
    }

    let mut cost_expr: Expression = 0.0.into();
    let mut idx = 0usize;
    for block in blocks {
        for comp in &get_candidates(db, &block.kind) {
            let price = comp.availability.as_ref().and_then(|a| a.price_1k).unwrap_or(0.0);
            cost_expr = cost_expr + x[idx] * price;
            idx += 1;
        }
    }

    let mut constraints: Vec<Constraint> = Vec::new();
    let mut start = 0usize;
    for block in blocks {
        let candidates = get_candidates(db, &block.kind);
        let mut sum_expr: Expression = 0.0.into();
        for j in 0..candidates.len() {
            sum_expr = sum_expr + x[start + j];
        }
        constraints.push(sum_expr.ge(1));
        start += candidates.len();
    }

    if budget > 0.0 {
        constraints.push(cost_expr.clone().leq(budget));
    }

    let problem = vars.minimise(cost_expr.clone());
    let mut with_c = problem.using(default_solver);
    for c in constraints {
        with_c = with_c.with(c);
    }

    let solved = with_c.solve().map_err(|e| format!("ILP solve failed: {}", e))?;
    let total_cost = solved.eval(&cost_expr);

    let mut assignments = HashMap::new();
    let mut scores = HashMap::new();
    let mut start2 = 0usize;
    for block in blocks {
        let candidates = get_candidates(db, &block.kind);
        for (j, comp) in candidates.iter().enumerate() {
            if solved.eval(&x[start2 + j]) > 0.5 {
                assignments.insert(block.id.clone(), comp.part.clone());
                scores.insert(block.id.clone(), score_component(comp, block));
            }
        }
        start2 += candidates.len();
    }
    Ok(Solution { assignments, scores, total_cost, feasible: true })
}

#[cfg(feature = "solver_z3")]
pub fn z3_solve(db: &ComponentDb, blocks: &[FunctionalBlock]) -> Result<Solution, String> {
    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let solver = z3::Solver::new(&ctx);
    let mut vars = Vec::new();

    for block in blocks {
        let candidates = get_candidates(db, &block.kind);
        if candidates.is_empty() {
            return Err(format!("No candidates for block {}", block.id));
        }
        let sym = z3::Symbol::from_string(&ctx, &format!("v_{}", block.id));
        let var = z3::ast::Int::new_const(&ctx, &sym);
        vars.push(var.clone());
        solver.assert(&var.ge(&z3::ast::Int::from_i64(&ctx, 0)));
        solver.assert(&var.lt(&z3::ast::Int::from_i64(&ctx, candidates.len() as i64)));

        if let Some(ref dma) = block.dma {
            if dma.required {
                let mut alt = Vec::new();
                for (i, comp) in candidates.iter().enumerate() {
                    let has_dma = comp.features.peripherals.get("dma").copied().unwrap_or(0) > 0;
                    if has_dma {
                        alt.push(var._eq(&z3::ast::Int::from_i64(&ctx, i as i64)));
                    }
                }
                if !alt.is_empty() {
                    solver.assert(&z3::ast::Bool::or(&ctx, &alt));
                }
            }
        }
    }

    let feasible = solver.check() == z3::SatResult::Sat;
    let mut assignments = HashMap::new();
    let mut scores = HashMap::new();
    let mut total_cost = 0.0;

    if feasible {
        if let Some(model) = solver.get_model() {
            for (i, block) in blocks.iter().enumerate() {
                if let Some(val) = model.eval(&vars[i], true) {
                    if let Some(idx) = val.as_i64() {
                        let candidates = get_candidates(db, &block.kind);
                        if idx >= 0 && (idx as usize) < candidates.len() {
                            let comp = &candidates[idx as usize];
                            assignments.insert(block.id.clone(), comp.part.clone());
                            scores.insert(block.id.clone(), score_component(comp, block));
                            total_cost += comp.availability.as_ref().and_then(|a| a.price_1k).unwrap_or(0.0);
                        }
                    }
                }
            }
        }
    }
    Ok(Solution { assignments, scores, total_cost, feasible })
}

pub fn heuristic_solve(db: &ComponentDb, blocks: &[FunctionalBlock]) -> Solution {
    let mut assignments = HashMap::new();
    let mut scores = HashMap::new();
    let mut total_cost = 0.0;

    for block in blocks {
        let candidates = get_candidates(db, &block.kind);
        if let Some(best) = candidates.iter()
            .map(|c| (score_component(c, block), c))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        {
            assignments.insert(block.id.clone(), best.1.part.clone());
            scores.insert(block.id.clone(), best.0);
            total_cost += best.1.availability.as_ref().and_then(|a| a.price_1k).unwrap_or(0.0);
        }
    }
    Solution { assignments, scores, total_cost, feasible: true }
}

fn get_candidates(db: &ComponentDb, kind: &BlockKind) -> Vec<ComponentEntry> {
    let entries = match kind {
        BlockKind::Gpu | BlockKind::Dma | BlockKind::Audio | BlockKind::Spi
        | BlockKind::I2c | BlockKind::Uart | BlockKind::Usb | BlockKind::Timer
        | BlockKind::InterruptController => db.by_category(ComponentCategory::Mcu),
        BlockKind::Ethernet => db.by_category(ComponentCategory::Connectivity),
        BlockKind::MemoryController => db.by_category(ComponentCategory::Memory),
        _ => db.by_category(ComponentCategory::Mcu),
    };
    entries.into_iter().cloned().collect()
}

fn score_component(comp: &ComponentEntry, block: &FunctionalBlock) -> f64 {
    let mut score = 0.0;
    let mut factors = 0u32;
    if category_matches(&comp.category, &block.kind) { score += 0.3; factors += 1; }
    let required = required_peripherals(&block.kind);
    if !required.is_empty() {
        let found = required.iter().filter(|p| comp.features.peripherals.contains_key(**p)).count();
        score += (found as f64 / required.len() as f64) * 0.3;
        factors += 1;
    }
    if let Some(ref dma) = block.dma {
        if dma.required && comp.features.peripherals.get("dma").copied().unwrap_or(0) > 0 {
            score += 0.2; factors += 1;
        }
    }
    if let Some(ref cpu) = comp.features.cpu {
        if cpu.max_mhz >= 100 { score += 0.2; factors += 1; }
    }
    if factors == 0 { 0.3 } else { score / factors as f64 }
}

fn category_matches(cat: &ComponentCategory, kind: &BlockKind) -> bool {
    matches!((cat, kind),
        (ComponentCategory::Mcu, BlockKind::Gpu | BlockKind::Dma | BlockKind::Audio
            | BlockKind::Spi | BlockKind::I2c | BlockKind::Uart | BlockKind::Usb
            | BlockKind::Timer | BlockKind::InterruptController)
        | (ComponentCategory::Connectivity, BlockKind::Ethernet)
        | (ComponentCategory::Memory, BlockKind::MemoryController)
    )
}

fn required_peripherals(kind: &BlockKind) -> Vec<&'static str> {
    match kind {
        BlockKind::Gpu => vec!["spi", "dma"],
        BlockKind::Audio => vec!["i2c"],
        BlockKind::Dma => vec!["dma"],
        BlockKind::Usb => vec!["usb"],
        BlockKind::Ethernet => vec!["spi"],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_db() -> ComponentDb {
        let mut db = ComponentDb::new();
        db.add_entry(ComponentEntry {
            part: "RP2350A".into(), manufacturer: "RPi".into(), description: "MCU".into(),
            category: ComponentCategory::Mcu, package: Some("QFN-56".into()),
            features: crate::component_db::ComponentFeatures {
                cpu: Some(crate::component_db::CpuFeature { cores: 4, max_mhz: 150, architecture: None }),
                memory: None,
                peripherals: { let mut p = HashMap::new(); p.insert("dma".into(), 8); p.insert("spi".into(), 2); p },
            },
            timing: None, compatible_with: vec![],
            power: None, pins: None, availability: Some(crate::component_db::Availability {
                status: "production".into(), price_1k: Some(1.50), distributor: vec![],
            }),
        });
        db.add_entry(ComponentEntry {
            part: "RK3566".into(), manufacturer: "Rockchip".into(), description: "SoC".into(),
            category: ComponentCategory::Cpu, package: Some("BGA-364".into()),
            features: crate::component_db::ComponentFeatures {
                cpu: Some(crate::component_db::CpuFeature { cores: 4, max_mhz: 2000, architecture: None }),
                memory: None,
                peripherals: { let mut p = HashMap::new(); p.insert("dma".into(), 16); p.insert("spi".into(), 4); p },
            },
            timing: None, compatible_with: vec![],
            power: None, pins: None, availability: Some(crate::component_db::Availability {
                status: "production".into(), price_1k: Some(12.00), distributor: vec![],
            }),
        });
        db
    }

    fn mock_block() -> FunctionalBlock {
        FunctionalBlock {
            id: "gpu_0".into(), kind: BlockKind::Gpu,
            base_address: 0x10000000, size: 0x1000,
            registers: vec![], protocol: crate::spec::types::Protocol {
                states: vec![], transitions: vec![], entry_condition: None, exit_condition: None,
            },
            timing: crate::spec::types::TimingProfile {
                activation: None, processing: None, interrupt_response: None, dma_setup: None, polling_interval: None,
            },
            dma: Some(DmaRequirement { required: true, min_bandwidth_mbps: 100.0, alignment: 4, max_channels: 2 }),
            dependencies: vec![], confidence: 0.8,
        }
    }

    #[test]
    fn test_heuristic_solve() {
        let db = mock_db();
        let sol = heuristic_solve(&db, &[mock_block()]);
        assert!(sol.feasible);
        assert_eq!(sol.assignments.get("gpu_0").unwrap(), "RP2350A");
    }

    #[test]
    fn test_solve_fallback() {
        let db = mock_db();
        let sol = solve(&db, &[mock_block()], 100.0);
        assert!(sol.feasible);
    }

    #[test]
    fn test_ilp_with_feature() {
        #[cfg(feature = "solver_ilp")]
        {
            let db = mock_db();
            let result = ilp_optimize(&db, &[mock_block()], 50.0);
            assert!(result.is_ok());
        }
    }
}
