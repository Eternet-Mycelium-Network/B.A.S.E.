/// Constraint Solver — engine de otimização para seleção de componentes.
///
/// Abordagem híbrida:
/// 1. Z3 (SMT) resolve constraints lógicas (tem que ter DMA, potência ≤ 5W)
/// 2. good_lp (ILP) otimiza custo dentro do espaço de soluções factíveis
pub mod heuristic;
