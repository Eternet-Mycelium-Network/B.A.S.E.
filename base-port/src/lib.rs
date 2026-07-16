//! Port package — mapa de endereços/drivers, inventário de fósseis, atlas MD.
//!
//! **Não** reescreve o OS completo. Gera o que o engenheiro precisa para *não*
//! começar do zero ao mapear HAL/drivers entre arquiteturas.

mod fossils;
mod map;
mod package;

pub use fossils::{FossilInventory, FossilKind, FossilRecord};
pub use map::{AddressDriverMap, MappedRegion, TranslationStrategy};
pub use package::{build_port_package, PortPackage, PortPackageOptions};
