/// Tensão Ψ — métrica formal de confiança baseada na Paleocomputação Estrutural.
///
/// Ψ(B, H) = ∫ δ(ω_obs, ω_H) dμ
///
/// Onde:
///   δ = divergência local entre observáveis
///   dμ = peso do bloco (importância)
///   Ψ ∈ [0, ∞): 0 = reconstrução perfeita, ∞ = sem correspondência
///
/// confidence = max(0, 1 - Ψ_normalized)
/// Ψ_normalized = Ψ / (1 + Ψ) ∈ [0, 1)
use crate::evidence::{EvidenceDb, EvidenceType, EvidenceEntry};
use crate::spec::types::{HardwareSpec, FunctionalBlock, Register, TimingProfile};

/// Resultado completo da análise de tensão
#[derive(Debug, Clone)]
pub struct TensionReport {
    pub overall_tension: f64,
    pub overall_confidence: f64,
    pub compilatory_entropy: f64,
    pub block_tensions: Vec<BlockTension>,
}

#[derive(Debug, Clone)]
pub struct BlockTension {
    pub block_id: String,
    pub tension: f64,
    pub confidence: f64,
    pub components: TensionComponents,
}

#[derive(Debug, Clone)]
pub struct TensionComponents {
    pub register_divergence: f64,     // δ_reg: divergência de registradores
    pub access_divergence: f64,       // δ_acc: divergência de padrões de acesso
    pub timing_divergence: f64,       // δ_tim: divergência de timing
    pub structural_divergence: f64,   // δ_str: divergência estrutural (CFG)
    pub block_weight: f64,            // dμ: importância do bloco
}

/// Calculador de tensão Ψ
pub struct TensionMetric;

impl TensionMetric {
    /// Calcula a tensão entre Evidence DB e HardwareSpec
    pub fn compute(evidence: &EvidenceDb, spec: &HardwareSpec) -> TensionReport {
        let total_evidence = evidence.count() as f64;
        
        // 1. Compilatory entropy S(B) = log|C⁻¹(B)|
        //    Simplified: S = log(possible_programs)
        //    We estimate from total_evidence / unique_addresses
        let unique_addrs = evidence.unique_mmio_addresses().len() as f64;
        let compilatory_entropy = if unique_addrs > 0.0 {
            (total_evidence / unique_addrs).ln()
        } else { 0.0 };

        let mut block_tensions = Vec::new();
        let mut total_tension = 0.0;
        let mut total_weight = 0.0;

        for block in &spec.blocks {
            let bt = Self::compute_block_tension(block, evidence);
            total_tension += bt.tension * bt.components.block_weight;
            total_weight += bt.components.block_weight;
            block_tensions.push(bt);
        }

        let overall_tension = if total_weight > 0.0 {
            total_tension / total_weight
        } else {
            compilatory_entropy.min(1.0) // fallback: use entropy
        };

        // Normalize: Ψ_normalized = Ψ / (1 + Ψ)
        let psi_norm = overall_tension / (1.0 + overall_tension);
        let overall_confidence = (1.0 - psi_norm).max(0.0).min(1.0);

        TensionReport {
            overall_tension,
            overall_confidence,
            compilatory_entropy,
            block_tensions,
        }
    }

    /// Calcula tensão para um bloco específico
    fn compute_block_tension(block: &FunctionalBlock, evidence: &EvidenceDb) -> BlockTension {
        // Filter evidence relevant to this block's address range
        let relevant: Vec<&EvidenceEntry> = evidence.entries.iter().filter(|e| {
            match &e.evidence_type {
                EvidenceType::MmioWrite { address, .. } | EvidenceType::MmioRead { address } => {
                    *address >= block.base_address && *address < block.base_address + block.size
                }
                _ => false,
            }
        }).collect();

        // δ_reg: register divergence — model has registers for observed addresses?
        let observed_addrs: Vec<u64> = relevant.iter().filter_map(|e| {
            match e.evidence_type {
                EvidenceType::MmioWrite { address, .. } | EvidenceType::MmioRead { address } => Some(address),
                _ => None,
            }
        }).collect();

        let addr_count = observed_addrs.len() as f64;
        let reg_count = block.registers.len() as f64;

        let register_divergence = if addr_count > 0.0 {
            let covered = block.registers.iter().filter(|r| {
                let addr = block.base_address + r.offset as u64;
                observed_addrs.contains(&addr)
            }).count() as f64;
            1.0 - (covered / addr_count)
        } else { 0.5 }; // neutral if no evidence

        // δ_acc: access pattern divergence — write vs read ratio
        let writes = relevant.iter().filter(|e| {
            matches!(e.evidence_type, EvidenceType::MmioWrite { .. })
        }).count() as f64;

        let reads = relevant.iter().filter(|e| {
            matches!(e.evidence_type, EvidenceType::MmioRead { .. })
        }).count() as f64;

        let total_acc = writes + reads;
        let access_divergence = if total_acc > 0.0 {
            let obs_ratio = writes / total_acc;
            let reg_writes = block.registers.iter().filter(|r| matches!(r.access, crate::spec::types::AccessType::Write | crate::spec::types::AccessType::ReadWrite)).count() as f64;
            let reg_total = block.registers.len() as f64;
            let model_ratio = if reg_total > 0.0 { reg_writes / reg_total } else { 0.5 };
            (obs_ratio - model_ratio).abs()
        } else { 0.0 };

        // δ_tim: timing divergence — model has timing for observed operations?
        let timing_divergence = if block.timing.activation.is_some() || block.timing.processing.is_some() {
            if relevant.len() > 5 { 0.2 } else { 0.5 } // more evidence = more confidence in timing
        } else { 0.8 }; // no timing at all = high divergence

        // δ_str: structural divergence — model structure matches expectations
        let structural_divergence = if block.registers.is_empty() { 0.9 }
            else if block.confidence < 0.3 { 0.7 }
            else if block.confidence < 0.6 { 0.4 }
            else { 0.15 };

        // dμ: block weight — importance based on register count and evidence volume
        let block_weight = 0.5 + 0.5 * (1.0 - (-reg_count / 10.0).exp()) * (1.0 + (relevant.len() as f64 / 20.0).min(1.0));

        // Combine: Ψ_block = Σ w_i * δ_i / Σ w_i
        // We use equal weights for components
        let components = TensionComponents {
            register_divergence,
            access_divergence,
            timing_divergence,
            structural_divergence,
            block_weight,
        };

        let raw_tension = (register_divergence + access_divergence + timing_divergence + structural_divergence) / 4.0;

        // Normalize block tension
        let tension = raw_tension / (1.0 + raw_tension);
        let confidence = (1.0 - tension).max(0.0).min(1.0);

        BlockTension {
            block_id: block.id.clone(),
            tension,
            confidence,
            components,
        }
    }

    /// Entropia compilatória S(B) = log|C⁻¹(B)|
    ///
    /// Mede quantos programas fonte poderiam produzir este binário.
    /// Alta entropia = muitos programas possíveis = baixa confiança.
    pub fn compilatory_entropy(evidence: &EvidenceDb, spec: &HardwareSpec) -> f64 {
        let total = evidence.count() as f64;
        let unique = evidence.unique_mmio_addresses().len() as f64;
        let blocks = spec.blocks.len() as f64;

        if total > 0.0 && unique > 0.0 && blocks > 0.0 {
            // S = log(evidence_per_address * block_complexity)
            let evidence_density = total / unique;
            let block_complexity = blocks.sqrt();
            (evidence_density * block_complexity).ln().max(0.0)
        } else { 0.0 }
    }

    /// Converte tensão em confidence score legado
    pub fn tension_to_confidence(tension: f64) -> f64 {
        // Ψ_normalized = Ψ / (1 + Ψ) ∈ [0, 1)
        // confidence = 1 - Ψ_normalized
        let psi_norm = tension / (1.0 + tension);
        (1.0 - psi_norm).max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::*;
    use crate::spec::types::{self, *};
    use std::collections::HashMap;

    fn sample_evidence() -> EvidenceDb {
        let mut db = EvidenceDb::new("test");
        db.add(EvidenceEntry {
            id: "ev_001".into(),
            evidence_type: EvidenceType::MmioWrite { address: 0x10000000, value: Some(1) },
            context: [("func".into(), "init".into())].into(),
        });
        db.add(EvidenceEntry {
            id: "ev_002".into(),
            evidence_type: EvidenceType::MmioWrite { address: 0x10000004, value: Some(0) },
            context: HashMap::new(),
        });
        db.add(EvidenceEntry {
            id: "ev_003".into(),
            evidence_type: EvidenceType::MmioRead { address: 0x10000004 },
            context: HashMap::new(),
        });
        db
    }

    fn sample_spec() -> HardwareSpec {
        let mut spec = HardwareSpec::empty();
        spec.blocks.push(FunctionalBlock {
            id: "gpu_0".into(), kind: BlockKind::Gpu,
            base_address: 0x10000000, size: 0x1000,
            registers: vec![
                Register { offset: 0, name: Some("control".into()), width: 32,
                    access: AccessType::ReadWrite, purpose: RegisterPurpose::Control,
                    reset_value: None, observed_values: vec![], bitfields: vec![], polling: false, count: 0,
                },
                Register { offset: 4, name: Some("status".into()), width: 32,
                    access: AccessType::Read, purpose: RegisterPurpose::Status,
                    reset_value: None, observed_values: vec![], bitfields: vec![], polling: false, count: 0,
                },
            ],
            protocol: Protocol { states: vec![], transitions: vec![], entry_condition: None, exit_condition: None },
            timing: TimingProfile {
                activation: Some(LatencyRange::new(100, 500, 300)),
                processing: None, interrupt_response: None, dma_setup: None, polling_interval: None,
            },
            dma: None, dependencies: vec![], confidence: 0.8,
        });
        spec
    }

    #[test]
    fn test_tension_compute() {
        let evidence = sample_evidence();
        let spec = sample_spec();
        let report = TensionMetric::compute(&evidence, &spec);
        assert!(report.overall_tension >= 0.0);
        assert!(report.overall_confidence >= 0.0 && report.overall_confidence <= 1.0);
        assert!(!report.block_tensions.is_empty());
    }

    #[test]
    fn test_perfect_match_low_tension() {
        // Evidence perfectly matches model → low tension
        let mut evidence = EvidenceDb::new("test");
        evidence.add(EvidenceEntry {
            id: "ev_001".into(),
            evidence_type: EvidenceType::MmioWrite { address: 0x10000000, value: Some(1) },
            context: HashMap::new(),
        });
        let mut spec = HardwareSpec::empty();
        spec.blocks.push(FunctionalBlock {
            id: "test".into(), kind: BlockKind::Unknown,
            base_address: 0x10000000, size: 0x1000,
            registers: vec![Register {
                offset: 0, name: Some("ctrl".into()), width: 32,
                access: AccessType::Write, purpose: RegisterPurpose::Control,
                reset_value: None, observed_values: vec![], bitfields: vec![], polling: false, count: 0,
            }],
            protocol: Protocol { states: vec![], transitions: vec![], entry_condition: None, exit_condition: None },
            timing: TimingProfile {
                activation: Some(LatencyRange::new(100, 500, 300)),
                processing: None, interrupt_response: None, dma_setup: None, polling_interval: None,
            },
            dma: None, dependencies: vec![], confidence: 0.9,
        });
        let report = TensionMetric::compute(&evidence, &spec);
        assert!(report.overall_confidence > 0.3, "Good match should have reasonable confidence");
    }

    #[test]
    fn test_compilatory_entropy() {
        let evidence = sample_evidence();
        let spec = sample_spec();
        let entropy = TensionMetric::compilatory_entropy(&evidence, &spec);
        assert!(entropy >= 0.0);
    }

    #[test]
    fn test_tension_to_confidence() {
        assert!(TensionMetric::tension_to_confidence(0.0) > 0.9);
        assert!(TensionMetric::tension_to_confidence(10.0) < 0.2);
        assert!(TensionMetric::tension_to_confidence(1.0) > 0.0 && TensionMetric::tension_to_confidence(1.0) < 1.0);
    }

    #[test]
    fn test_block_tension_components() {
        let evidence = sample_evidence();
        let spec = sample_spec();
        let report = TensionMetric::compute(&evidence, &spec);
        let bt = &report.block_tensions[0];
        assert!(bt.components.register_divergence >= 0.0);
        assert!(bt.components.access_divergence >= 0.0);
        assert!(bt.components.block_weight > 0.0);
    }

    #[test]
    fn test_empty_evidence() {
        let evidence = EvidenceDb::new("empty");
        let spec = sample_spec();
        let report = TensionMetric::compute(&evidence, &spec);
        // Without evidence, tension should be moderate (neutral)
        assert!(report.overall_tension >= 0.0);
    }
}
