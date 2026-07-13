/// Solver heurístico baseado em score — usado quando Z3 não está disponível.
/// Substitui o matching simples por uma análise multi-fator.
use crate::component_db::{ComponentCategory, ComponentDb, ComponentEntry};
use crate::spec::types::{BlockKind, FunctionalBlock, SystemConstraints};

#[derive(Debug, Clone)]
pub struct HeuristicSolution {
    pub component: ComponentEntry,
    pub score: f64,
    pub reasons: Vec<String>,
}

/// Pontua um componente para um bloco específico
pub fn score_component(component: &ComponentEntry, block: &FunctionalBlock) -> HeuristicSolution {
    let mut score = 0.0f64;
    let mut reasons = Vec::new();

    // Fator 1: Categoria
    let cat_match = match block.kind {
        BlockKind::Gpu | BlockKind::Dma | BlockKind::Audio |
        BlockKind::Spi | BlockKind::I2c | BlockKind::Uart |
        BlockKind::Usb | BlockKind::Timer | BlockKind::InterruptController => {
            component.category == ComponentCategory::Mcu
        }
        BlockKind::Ethernet => component.category == ComponentCategory::Connectivity,
        BlockKind::MemoryController => component.category == ComponentCategory::Memory,
        _ => true,
    };
    if cat_match {
        score += 0.3;
        reasons.push(format!("category mismatch → -0.3"));
    }

    // Fator 2: Periféricos requeridos
    let required_periphs = required_peripherals(&block.kind);
    let mut found_periphs = 0u32;
    for periph in &required_periphs {
        if component.features.peripherals.contains_key(periph) {
            found_periphs += 1;
        }
    }
    if !required_periphs.is_empty() {
        let ratio = found_periphs as f64 / required_periphs.len() as f64;
        score += ratio * 0.3;
        if ratio > 0.0 {
            reasons.push(format!("peripherals: {}/{}", found_periphs, required_periphs.len()));
        }
    }

    // Fator 3: DMA
    if let Some(ref dma) = block.dma {
        if dma.required {
            let has_dma = component.features.peripherals.get("dma").copied().unwrap_or(0) > 0;
            if has_dma {
                score += 0.2;
                reasons.push("has dma".into());
            }
        }
    }

    // Fator 4: GPIO
    if matches!(block.kind, BlockKind::Gpu | BlockKind::Dma) {
        let gpio_count = component.pins.as_ref().map_or(0, |p| p.len() as u32);
        if gpio_count >= 16 {
            score += 0.1;
            reasons.push("gpio≥16".into());
        }
    }

    // Fator 5: Clock speed
    if let Some(ref cpu) = component.features.cpu {
        if cpu.max_mhz > 100 {
            score += 0.1;
            reasons.push(format!("cpu@{}MHz", cpu.max_mhz));
        }
    }

    HeuristicSolution { component: component.clone(), score, reasons }
}

/// Encontra o melhor componente para um bloco no banco de dados
pub fn find_best_component(db: &ComponentDb, block: &FunctionalBlock) -> Option<HeuristicSolution> {
    let candidates = match block.kind {
        BlockKind::Gpu | BlockKind::Dma | BlockKind::Audio | BlockKind::Spi
        | BlockKind::I2c | BlockKind::Uart | BlockKind::Usb | BlockKind::Timer
        | BlockKind::InterruptController => db.by_category(ComponentCategory::Mcu),
        BlockKind::Ethernet => db.by_category(ComponentCategory::Connectivity),
        BlockKind::MemoryController => db.by_category(ComponentCategory::Memory),
        _ => db.by_category(ComponentCategory::Mcu),
    };

    candidates.iter()
        .map(|c| score_component(c, block))
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
}

fn required_peripherals(kind: &BlockKind) -> Vec<String> {
    match kind {
        BlockKind::Gpu => vec!["spi".into(), "dma".into()],
        BlockKind::Audio => vec!["i2c".into()],
        BlockKind::Dma => vec!["dma".into()],
        BlockKind::Usb => vec!["usb".into()],
        BlockKind::Ethernet => vec!["spi".into()],
        BlockKind::Spi => vec!["spi".into()],
        BlockKind::I2c => vec!["i2c".into()],
        BlockKind::Uart => vec!["uart".into()],
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
            part: "RP2350A".into(),
            manufacturer: "RPi".into(),
            description: "MCU".into(),
            category: ComponentCategory::Mcu,
            package: Some("QFN-56".into()),
            features: crate::component_db::ComponentFeatures {
                cpu: Some(crate::component_db::CpuFeature { cores: 4, max_mhz: 150, architecture: None }),
                memory: None,
                peripherals: {
                    let mut p = HashMap::new();
                    p.insert("dma".into(), 8);
                    p.insert("spi".into(), 2);
                    p.insert("i2c".into(), 2);
                    p
                },
            },
            timing: None, compatible_with: vec![],
            power: None, pins: None, availability: None,
        });
        db
    }

    #[test]
    fn test_score_component() {
        let db = mock_db();
        let comp = db.by_name("RP2350A").unwrap();
        let block = FunctionalBlock {
            id: "gpu_0".into(), kind: BlockKind::Gpu,
            base_address: 0x10000000, size: 0x1000,
            registers: vec![],
            protocol: crate::spec::types::Protocol {
                states: vec![], transitions: vec![], entry_condition: None, exit_condition: None,
            },
            timing: crate::spec::types::TimingProfile {
                activation: None, processing: None, interrupt_response: None, dma_setup: None, polling_interval: None,
            },
            dma: Some(crate::spec::types::DmaRequirement { required: true, min_bandwidth_mbps: 100.0, alignment: 4, max_channels: 2 }),
            dependencies: vec![], confidence: 0.8,
        };
        let solution = score_component(comp, &block);
        assert!(solution.score > 0.3);
        assert!(!solution.reasons.is_empty());
    }

    #[test]
    fn test_find_best_component() {
        let db = mock_db();
        let block = FunctionalBlock {
            id: "dma_0".into(), kind: BlockKind::Dma,
            base_address: 0x20000000, size: 0x1000,
            registers: vec![],
            protocol: crate::spec::types::Protocol {
                states: vec![], transitions: vec![], entry_condition: None, exit_condition: None,
            },
            timing: crate::spec::types::TimingProfile {
                activation: None, processing: None, interrupt_response: None, dma_setup: None, polling_interval: None,
            },
            dma: Some(crate::spec::types::DmaRequirement { required: true, min_bandwidth_mbps: 400.0, alignment: 256, max_channels: 4 }),
            dependencies: vec![], confidence: 0.8,
        };
        let best = find_best_component(&db, &block);
        assert!(best.is_some());
        assert_eq!(best.unwrap().component.part, "RP2350A");
    }
}
