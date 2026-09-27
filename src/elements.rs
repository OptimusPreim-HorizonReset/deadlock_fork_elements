#![allow(dead_code, clippy::excessive_precision)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementFamily {
    AlkaliMetal,
    AlkalineEarthMetal,
    TransitionMetal,
    PostTransitionMetal,
    Metalloid,
    ReactiveNonmetal,
    Halogen,
    NobleGas,
    Lanthanoid,
    Actinoid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingTendency {
    Inert,
    Metallic,
    NonPolarCovalent,
    PolarCovalent,
    Ionic,
}

#[derive(Clone, Copy, Debug)]
struct ElementRecord {
    atomic_number: u8,
    symbol: &'static str,
    name: &'static str,
    atomic_weight: f32,
    period: u8,
    group: u8,
    pauling_electronegativity: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct ElementDefinition {
    pub atomic_number: u8,
    pub symbol: &'static str,
    pub name: &'static str,
    pub atomic_weight: f32,
    pub atomic_weight_is_estimated: bool,
    pub period: u8,
    pub group: u8,
    pub family: ElementFamily,
    pub pauling_electronegativity: Option<f32>,
    pub effective_electronegativity: f32,
    pub electronegativity_is_estimated: bool,
    pub covalent_radius_pm: f32,
    pub radius_is_estimated: bool,
}

macro_rules! element_record {
    ($atomic_number:literal, $symbol:literal, $name:literal, $weight:literal, $period:literal, $group:literal, $electronegativity:expr) => {
        ElementRecord {
            atomic_number: $atomic_number,
            symbol: $symbol,
            name: $name,
            atomic_weight: $weight,
            period: $period,
            group: $group,
            pauling_electronegativity: $electronegativity,
        }
    };
}

const ELEMENT_RECORDS: [ElementRecord; 118] = [
    element_record!(1, "H", "Hydrogen", 1.008, 1, 1, Some(2.2)),
    element_record!(2, "He", "Helium", 4.002602, 1, 18, None),
    element_record!(3, "Li", "Lithium", 6.94, 2, 1, Some(0.98)),
    element_record!(4, "Be", "Beryllium", 9.012183, 2, 2, Some(1.57)),
    element_record!(5, "B", "Boron", 10.81, 2, 13, Some(2.04)),
    element_record!(6, "C", "Carbon", 12.011, 2, 14, Some(2.55)),
    element_record!(7, "N", "Nitrogen", 14.007, 2, 15, Some(3.04)),
    element_record!(8, "O", "Oxygen", 15.999, 2, 16, Some(3.44)),
    element_record!(9, "F", "Fluorine", 18.998403, 2, 17, Some(3.98)),
    element_record!(10, "Ne", "Neon", 20.17976, 2, 18, None),
    element_record!(11, "Na", "Sodium", 22.989769, 3, 1, Some(0.93)),
    element_record!(12, "Mg", "Magnesium", 24.305, 3, 2, Some(1.31)),
    element_record!(13, "Al", "Aluminium", 26.981539, 3, 13, Some(1.61)),
    element_record!(14, "Si", "Silicon", 28.085, 3, 14, Some(1.9)),
    element_record!(15, "P", "Phosphorus", 30.973762, 3, 15, Some(2.19)),
    element_record!(16, "S", "Sulfur", 32.06, 3, 16, Some(2.58)),
    element_record!(17, "Cl", "Chlorine", 35.45, 3, 17, Some(3.16)),
    element_record!(18, "Ar", "Argon", 39.9481, 3, 18, None),
    element_record!(19, "K", "Potassium", 39.09831, 4, 1, Some(0.82)),
    element_record!(20, "Ca", "Calcium", 40.0784, 4, 2, Some(1.0)),
    element_record!(21, "Sc", "Scandium", 44.955909, 4, 3, Some(1.36)),
    element_record!(22, "Ti", "Titanium", 47.8671, 4, 4, Some(1.54)),
    element_record!(23, "V", "Vanadium", 50.94151, 4, 5, Some(1.63)),
    element_record!(24, "Cr", "Chromium", 51.99616, 4, 6, Some(1.66)),
    element_record!(25, "Mn", "Manganese", 54.938044, 4, 7, Some(1.55)),
    element_record!(26, "Fe", "Iron", 55.8452, 4, 8, Some(1.83)),
    element_record!(27, "Co", "Cobalt", 58.933194, 4, 9, Some(1.88)),
    element_record!(28, "Ni", "Nickel", 58.69344, 4, 10, Some(1.91)),
    element_record!(29, "Cu", "Copper", 63.5463, 4, 11, Some(1.9)),
    element_record!(30, "Zn", "Zinc", 65.382, 4, 12, Some(1.65)),
    element_record!(31, "Ga", "Gallium", 69.7231, 4, 13, Some(1.81)),
    element_record!(32, "Ge", "Germanium", 72.6308, 4, 14, Some(2.01)),
    element_record!(33, "As", "Arsenic", 74.921596, 4, 15, Some(2.18)),
    element_record!(34, "Se", "Selenium", 78.9718, 4, 16, Some(2.55)),
    element_record!(35, "Br", "Bromine", 79.904, 4, 17, Some(2.96)),
    element_record!(36, "Kr", "Krypton", 83.7982, 4, 18, Some(3.0)),
    element_record!(37, "Rb", "Rubidium", 85.46783, 5, 1, Some(0.82)),
    element_record!(38, "Sr", "Strontium", 87.621, 5, 2, Some(0.95)),
    element_record!(39, "Y", "Yttrium", 88.905842, 5, 3, Some(1.22)),
    element_record!(40, "Zr", "Zirconium", 91.2242, 5, 4, Some(1.33)),
    element_record!(41, "Nb", "Niobium", 92.906372, 5, 5, Some(1.6)),
    element_record!(42, "Mo", "Molybdenum", 95.951, 5, 6, Some(2.16)),
    element_record!(43, "Tc", "Technetium", 98.0, 5, 7, Some(1.9)),
    element_record!(44, "Ru", "Ruthenium", 101.072, 5, 8, Some(2.2)),
    element_record!(45, "Rh", "Rhodium", 102.905502, 5, 9, Some(2.28)),
    element_record!(46, "Pd", "Palladium", 106.421, 5, 10, Some(2.2)),
    element_record!(47, "Ag", "Silver", 107.86822, 5, 11, Some(1.93)),
    element_record!(48, "Cd", "Cadmium", 112.4144, 5, 12, Some(1.69)),
    element_record!(49, "In", "Indium", 114.8181, 5, 13, Some(1.78)),
    element_record!(50, "Sn", "Tin", 118.7107, 5, 14, Some(1.96)),
    element_record!(51, "Sb", "Antimony", 121.7601, 5, 15, Some(2.05)),
    element_record!(52, "Te", "Tellurium", 127.603, 5, 16, Some(2.1)),
    element_record!(53, "I", "Iodine", 126.904473, 5, 17, Some(2.66)),
    element_record!(54, "Xe", "Xenon", 131.2936, 5, 18, Some(2.6)),
    element_record!(55, "Cs", "Cesium", 132.905452, 6, 1, Some(0.79)),
    element_record!(56, "Ba", "Barium", 137.3277, 6, 2, Some(0.89)),
    element_record!(57, "La", "Lanthanum", 138.905477, 6, 3, Some(1.1)),
    element_record!(58, "Ce", "Cerium", 140.1161, 6, 3, Some(1.12)),
    element_record!(59, "Pr", "Praseodymium", 140.907662, 6, 3, Some(1.13)),
    element_record!(60, "Nd", "Neodymium", 144.2423, 6, 3, Some(1.14)),
    element_record!(61, "Pm", "Promethium", 145.0, 6, 3, Some(1.13)),
    element_record!(62, "Sm", "Samarium", 150.362, 6, 3, Some(1.17)),
    element_record!(63, "Eu", "Europium", 151.9641, 6, 3, Some(1.2)),
    element_record!(64, "Gd", "Gadolinium", 157.253, 6, 3, Some(1.2)),
    element_record!(65, "Tb", "Terbium", 158.925352, 6, 3, Some(1.1)),
    element_record!(66, "Dy", "Dysprosium", 162.5001, 6, 3, Some(1.22)),
    element_record!(67, "Ho", "Holmium", 164.930332, 6, 3, Some(1.23)),
    element_record!(68, "Er", "Erbium", 167.2593, 6, 3, Some(1.24)),
    element_record!(69, "Tm", "Thulium", 168.934222, 6, 3, Some(1.25)),
    element_record!(70, "Yb", "Ytterbium", 173.0451, 6, 3, Some(1.1)),
    element_record!(71, "Lu", "Lutetium", 174.96681, 6, 3, Some(1.27)),
    element_record!(72, "Hf", "Hafnium", 178.492, 6, 4, Some(1.3)),
    element_record!(73, "Ta", "Tantalum", 180.947882, 6, 5, Some(1.5)),
    element_record!(74, "W", "Tungsten", 183.841, 6, 6, Some(2.36)),
    element_record!(75, "Re", "Rhenium", 186.2071, 6, 7, Some(1.9)),
    element_record!(76, "Os", "Osmium", 190.233, 6, 8, Some(2.2)),
    element_record!(77, "Ir", "Iridium", 192.2173, 6, 9, Some(2.2)),
    element_record!(78, "Pt", "Platinum", 195.0849, 6, 10, Some(2.28)),
    element_record!(79, "Au", "Gold", 196.96657, 6, 11, Some(2.54)),
    element_record!(80, "Hg", "Mercury", 200.5923, 6, 12, Some(2.0)),
    element_record!(81, "Tl", "Thallium", 204.38, 6, 13, Some(1.62)),
    element_record!(82, "Pb", "Lead", 207.21, 6, 14, Some(1.87)),
    element_record!(83, "Bi", "Bismuth", 208.980401, 6, 15, Some(2.02)),
    element_record!(84, "Po", "Polonium", 209.0, 6, 16, Some(2.0)),
    element_record!(85, "At", "Astatine", 210.0, 6, 17, Some(2.2)),
    element_record!(86, "Rn", "Radon", 222.0, 6, 18, Some(2.2)),
    element_record!(87, "Fr", "Francium", 223.0, 7, 1, Some(0.79)),
    element_record!(88, "Ra", "Radium", 226.0, 7, 2, Some(0.9)),
    element_record!(89, "Ac", "Actinium", 227.0, 7, 3, Some(1.1)),
    element_record!(90, "Th", "Thorium", 232.03774, 7, 3, Some(1.3)),
    element_record!(91, "Pa", "Protactinium", 231.035882, 7, 3, Some(1.5)),
    element_record!(92, "U", "Uranium", 238.028913, 7, 3, Some(1.38)),
    element_record!(93, "Np", "Neptunium", 237.0, 7, 3, Some(1.36)),
    element_record!(94, "Pu", "Plutonium", 244.0, 7, 3, Some(1.28)),
    element_record!(95, "Am", "Americium", 243.0, 7, 3, Some(1.13)),
    element_record!(96, "Cm", "Curium", 247.0, 7, 3, Some(1.28)),
    element_record!(97, "Bk", "Berkelium", 247.0, 7, 3, Some(1.3)),
    element_record!(98, "Cf", "Californium", 251.0, 7, 3, Some(1.3)),
    element_record!(99, "Es", "Einsteinium", 252.0, 7, 3, Some(1.3)),
    element_record!(100, "Fm", "Fermium", 257.0, 7, 3, Some(1.3)),
    element_record!(101, "Md", "Mendelevium", 258.0, 7, 3, Some(1.3)),
    element_record!(102, "No", "Nobelium", 259.0, 7, 3, Some(1.3)),
    element_record!(103, "Lr", "Lawrencium", 266.0, 7, 3, Some(1.3)),
    element_record!(104, "Rf", "Rutherfordium", 267.0, 7, 4, None),
    element_record!(105, "Db", "Dubnium", 268.0, 7, 5, None),
    element_record!(106, "Sg", "Seaborgium", 269.0, 7, 6, None),
    element_record!(107, "Bh", "Bohrium", 270.0, 7, 7, None),
    element_record!(108, "Hs", "Hassium", 269.0, 7, 8, None),
    element_record!(109, "Mt", "Meitnerium", 278.0, 7, 9, None),
    element_record!(110, "Ds", "Darmstadtium", 281.0, 7, 10, None),
    element_record!(111, "Rg", "Roentgenium", 282.0, 7, 11, None),
    element_record!(112, "Cn", "Copernicium", 285.0, 7, 12, None),
    element_record!(113, "Nh", "Nihonium", 286.0, 7, 13, None),
    element_record!(114, "Fl", "Flerovium", 289.0, 7, 14, None),
    element_record!(115, "Mc", "Moscovium", 289.0, 7, 15, None),
    element_record!(116, "Lv", "Livermorium", 293.0, 7, 16, None),
    element_record!(117, "Ts", "Tennessine", 294.0, 7, 17, None),
    element_record!(118, "Og", "Oganesson", 294.0, 7, 18, None),
];

impl ElementDefinition {
    pub fn color(self) -> [f32; 3] {
        let base = match self.family {
            ElementFamily::AlkaliMetal => [0.95, 0.30, 0.12],
            ElementFamily::AlkalineEarthMetal => [0.95, 0.76, 0.16],
            ElementFamily::TransitionMetal
            | ElementFamily::PostTransitionMetal
            | ElementFamily::Lanthanoid
            | ElementFamily::Actinoid => [0.16, 0.68, 0.38],
            ElementFamily::Metalloid => [0.12, 0.72, 0.72],
            ElementFamily::ReactiveNonmetal => [0.16, 0.43, 0.91],
            ElementFamily::Halogen => [0.34, 0.25, 0.78],
            ElementFamily::NobleGas => [0.67, 0.30, 0.80],
        };
        let shade = ((self.effective_electronegativity - 2.0) * 0.035
            + (self.atomic_number % 5) as f32 * 0.012)
            .clamp(-0.08, 0.08);

        [
            (base[0] + shade).clamp(0.0, 1.0),
            (base[1] + shade).clamp(0.0, 1.0),
            (base[2] + shade).clamp(0.0, 1.0),
        ]
    }

    pub fn reactivity_score(self) -> f32 {
        let period_offset = self.period.saturating_sub(2) as f32;
        let score = match self.family {
            ElementFamily::AlkaliMetal => 0.55 + period_offset * 0.075,
            ElementFamily::AlkalineEarthMetal => 0.38 + period_offset * 0.055,
            ElementFamily::TransitionMetal => {
                0.42 + (self.effective_electronegativity - 1.5) * 0.12
            }
            ElementFamily::PostTransitionMetal => {
                0.48 + (self.effective_electronegativity - 1.5) * 0.10
            }
            ElementFamily::Metalloid => 0.54,
            ElementFamily::ReactiveNonmetal => {
                0.48 + (self.effective_electronegativity - 2.0) * 0.12
            }
            ElementFamily::Halogen => 0.98 - period_offset * 0.075,
            ElementFamily::NobleGas => 0.02,
            ElementFamily::Lanthanoid => 0.40 + period_offset * 0.025,
            ElementFamily::Actinoid => 0.56 + period_offset * 0.02,
        };

        match self.atomic_number {
            1 => 0.45,
            8 => 0.92,
            9 => 1.0,
            54 => 0.08,
            55 => 0.96,
            86 => 0.10,
            87 => 0.98,
            118 => 0.12,
            _ => score.clamp(0.0, 1.0),
        }
    }

    pub fn core_suitability(self) -> f32 {
        let mass_score = (self.atomic_weight / 294.0).sqrt();
        let reactivity_penalty = self.reactivity_score() * 0.15;
        (mass_score - reactivity_penalty).clamp(0.0, 1.0)
    }

    pub fn scientific_gravity_scale(self, hydrogen_normalization: f32, weights: [f32; 4]) -> f32 {
        let hydrogen = element_by_atomic_number(1).expect("hydrogen is in the static catalog");
        let mass_log_ratio = (self.atomic_weight / hydrogen.atomic_weight).ln();
        let electronegativity_delta =
            (self.effective_electronegativity / hydrogen.effective_electronegativity - 1.0) * 0.25;
        let radius_log_ratio = (self.covalent_radius_pm / hydrogen.covalent_radius_pm).ln();
        let reactivity_log_ratio =
            ((self.reactivity_score() + 0.1) / (hydrogen.reactivity_score() + 0.1)).ln();
        let inert_gas_penalty = if self.family == ElementFamily::NobleGas {
            2.0
        } else {
            0.0
        };
        let log_scale = weights[0] * mass_log_ratio + weights[1] * electronegativity_delta
            - weights[2] * radius_log_ratio * 0.03
            + weights[3] * reactivity_log_ratio
            - inert_gas_penalty;

        (hydrogen_normalization.max(0.0) * log_scale.clamp(-8.0, 8.0).exp()).clamp(0.0, 1.0e6)
    }
}

pub fn element_by_atomic_number(atomic_number: u8) -> Option<ElementDefinition> {
    let record = *ELEMENT_RECORDS.get(atomic_number.checked_sub(1)? as usize)?;
    let family = family_for(record);
    let pauling_electronegativity = record.pauling_electronegativity;
    let effective_electronegativity =
        pauling_electronegativity.unwrap_or_else(|| estimate_electronegativity(record, family));

    Some(ElementDefinition {
        atomic_number: record.atomic_number,
        symbol: record.symbol,
        name: record.name,
        atomic_weight: record.atomic_weight,
        atomic_weight_is_estimated: matches!(record.atomic_number, 43 | 61 | 84..=118),
        period: record.period,
        group: record.group,
        family,
        pauling_electronegativity,
        effective_electronegativity,
        electronegativity_is_estimated: pauling_electronegativity.is_none(),
        covalent_radius_pm: estimate_covalent_radius(record),
        radius_is_estimated: true,
    })
}

pub fn all_elements() -> impl Iterator<Item = ElementDefinition> {
    (1..=118).filter_map(element_by_atomic_number)
}

pub fn binding_tendency(first: ElementDefinition, second: ElementDefinition) -> BindingTendency {
    if first.family == ElementFamily::NobleGas || second.family == ElementFamily::NobleGas {
        if first.reactivity_score() < 0.15 || second.reactivity_score() < 0.15 {
            return BindingTendency::Inert;
        }
    }

    let first_metal = is_metal(first.family);
    let second_metal = is_metal(second.family);
    if first_metal && second_metal {
        return BindingTendency::Metallic;
    }

    let electronegativity_difference =
        (first.effective_electronegativity - second.effective_electronegativity).abs();
    if electronegativity_difference >= 1.7 {
        BindingTendency::Ionic
    } else if electronegativity_difference >= 0.4 {
        BindingTendency::PolarCovalent
    } else {
        BindingTendency::NonPolarCovalent
    }
}

fn family_for(record: ElementRecord) -> ElementFamily {
    let atomic_number = record.atomic_number;
    if (57..=71).contains(&atomic_number) {
        return ElementFamily::Lanthanoid;
    }
    if (89..=103).contains(&atomic_number) {
        return ElementFamily::Actinoid;
    }
    if atomic_number == 1 {
        return ElementFamily::ReactiveNonmetal;
    }
    if record.group == 18 {
        return ElementFamily::NobleGas;
    }
    if record.group == 17 {
        return ElementFamily::Halogen;
    }
    if record.group == 1 {
        return ElementFamily::AlkaliMetal;
    }
    if record.group == 2 {
        return ElementFamily::AlkalineEarthMetal;
    }
    if matches!(atomic_number, 5 | 14 | 32 | 33 | 51 | 52 | 84 | 85) {
        return ElementFamily::Metalloid;
    }
    if record.group <= 12 {
        return ElementFamily::TransitionMetal;
    }
    if matches!(atomic_number, 6 | 7 | 8 | 15 | 16 | 34) {
        ElementFamily::ReactiveNonmetal
    } else {
        ElementFamily::PostTransitionMetal
    }
}

fn estimate_electronegativity(record: ElementRecord, family: ElementFamily) -> f32 {
    let period_offset = record.period.saturating_sub(2) as f32;
    match family {
        ElementFamily::AlkaliMetal => 0.95 - period_offset * 0.035,
        ElementFamily::AlkalineEarthMetal => 1.55 - period_offset * 0.08,
        ElementFamily::TransitionMetal => 1.45 + record.group as f32 * 0.055,
        ElementFamily::PostTransitionMetal => 1.50 + record.group.saturating_sub(13) as f32 * 0.12,
        ElementFamily::Metalloid => 2.0 + record.period as f32 * 0.025,
        ElementFamily::ReactiveNonmetal => 2.2 + record.group.saturating_sub(14) as f32 * 0.35,
        ElementFamily::Halogen => 3.2 - period_offset * 0.12,
        ElementFamily::NobleGas => 3.5,
        ElementFamily::Lanthanoid => 1.15 + period_offset * 0.01,
        ElementFamily::Actinoid => 1.25 + period_offset * 0.015,
    }
    .clamp(0.7, 4.0)
}

fn estimate_covalent_radius(record: ElementRecord) -> f32 {
    match record.atomic_number {
        1 => 31.0,
        2 => 28.0,
        _ => {
            let period_growth = (record.period as f32 - 4.0) * 14.0;
            let group_contraction = record.group.saturating_sub(1) as f32 * 5.5;
            (172.0 + period_growth - group_contraction).clamp(40.0, 250.0)
        }
    }
}

fn is_metal(family: ElementFamily) -> bool {
    matches!(
        family,
        ElementFamily::AlkaliMetal
            | ElementFamily::AlkalineEarthMetal
            | ElementFamily::TransitionMetal
            | ElementFamily::PostTransitionMetal
            | ElementFamily::Lanthanoid
            | ElementFamily::Actinoid
    )
}

#[cfg(test)]
mod tests {
    use super::{
        all_elements, binding_tendency, element_by_atomic_number, BindingTendency, ElementFamily,
    };

    #[test]
    fn catalog_contains_all_118_ordered_elements() {
        let elements: Vec<_> = all_elements().collect();
        assert_eq!(elements.len(), 118);
        assert!(elements.iter().enumerate().all(|(index, element)| {
            element.atomic_number as usize == index + 1
                && element.atomic_weight.is_finite()
                && element.atomic_weight > 0.0
                && element.covalent_radius_pm.is_finite()
                && (28.0..=250.0).contains(&element.covalent_radius_pm)
        }));
    }

    #[test]
    fn missing_electronegativities_are_explicitly_estimated() {
        let helium = element_by_atomic_number(2).unwrap();
        let fluorine = element_by_atomic_number(9).unwrap();
        let oganesson = element_by_atomic_number(118).unwrap();

        assert!(helium.electronegativity_is_estimated);
        assert!(!fluorine.electronegativity_is_estimated);
        assert!(oganesson.electronegativity_is_estimated);
        assert!(all_elements().all(|element| element.radius_is_estimated));
    }

    #[test]
    fn chemistry_families_match_documented_extremes() {
        assert_eq!(
            element_by_atomic_number(3).unwrap().family,
            ElementFamily::AlkaliMetal
        );
        assert_eq!(
            element_by_atomic_number(9).unwrap().family,
            ElementFamily::Halogen
        );
        assert_eq!(
            element_by_atomic_number(18).unwrap().family,
            ElementFamily::NobleGas
        );
        assert_eq!(
            element_by_atomic_number(26).unwrap().family,
            ElementFamily::TransitionMetal
        );
    }

    #[test]
    fn reactivity_extremes_and_binding_classes_are_bounded() {
        let fluorine = element_by_atomic_number(9).unwrap();
        let cesium = element_by_atomic_number(55).unwrap();
        let hydrogen = element_by_atomic_number(1).unwrap();
        let helium = element_by_atomic_number(2).unwrap();
        let neon = element_by_atomic_number(10).unwrap();

        assert_eq!(fluorine.reactivity_score(), 1.0);
        assert!(cesium.reactivity_score() > 0.9);
        assert!((0.0..=1.0).contains(&hydrogen.reactivity_score()));
        assert_eq!(binding_tendency(helium, neon), BindingTendency::Inert);
        assert_eq!(
            binding_tendency(hydrogen, hydrogen),
            BindingTendency::NonPolarCovalent
        );
    }

    #[test]
    fn element_colors_are_finite_and_bounded() {
        assert!(all_elements().all(|element| {
            element
                .color()
                .iter()
                .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
        }));
    }

    #[test]
    fn scientific_gravity_is_hydrogen_normalized_and_orders_heavy_elements() {
        let hydrogen = element_by_atomic_number(1).unwrap();
        let helium = element_by_atomic_number(2).unwrap();
        let uranium = element_by_atomic_number(92).unwrap();
        let weights = [1.0, 0.4, 10.0, 0.2];

        let hydrogen_scale = hydrogen.scientific_gravity_scale(0.06, weights);
        let helium_scale = helium.scientific_gravity_scale(0.06, weights);
        let uranium_scale = uranium.scientific_gravity_scale(0.06, weights);

        assert!((hydrogen_scale - 0.06).abs() < 1.0e-6);
        assert!(helium_scale < hydrogen_scale);
        assert!(uranium_scale > helium_scale);
    }
}
