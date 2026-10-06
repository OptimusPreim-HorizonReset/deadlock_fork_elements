#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::elements::BindingTendency;

#[derive(Clone, Copy, Debug)]
pub struct MolecularBond {
    pub first_body_id: u64,
    pub second_body_id: u64,
    pub rest_length: f32,
    pub tendency: BindingTendency,
}

#[derive(Clone, Copy, Debug)]
pub struct CompoundDefinition {
    pub name: &'static str,
    pub formula: &'static str,
    pub composition: &'static [(u8, u16)],
}

pub const KNOWN_COMPOUNDS: &[CompoundDefinition] = &[
    CompoundDefinition {
        name: "Hydrogen",
        formula: "H2",
        composition: &[(1, 2)],
    },
    CompoundDefinition {
        name: "Nitrogen",
        formula: "N2",
        composition: &[(7, 2)],
    },
    CompoundDefinition {
        name: "Oxygen",
        formula: "O2",
        composition: &[(8, 2)],
    },
    CompoundDefinition {
        name: "Fluorine",
        formula: "F2",
        composition: &[(9, 2)],
    },
    CompoundDefinition {
        name: "Chlorine",
        formula: "Cl2",
        composition: &[(17, 2)],
    },
    CompoundDefinition {
        name: "Bromine",
        formula: "Br2",
        composition: &[(35, 2)],
    },
    CompoundDefinition {
        name: "Iodine",
        formula: "I2",
        composition: &[(53, 2)],
    },
    CompoundDefinition {
        name: "Water",
        formula: "H2O",
        composition: &[(1, 2), (8, 1)],
    },
    CompoundDefinition {
        name: "Hydrogen chloride",
        formula: "HCl",
        composition: &[(1, 1), (17, 1)],
    },
    CompoundDefinition {
        name: "Sodium hydroxide",
        formula: "NaOH",
        composition: &[(1, 1), (8, 1), (11, 1)],
    },
    CompoundDefinition {
        name: "Sulfuric acid",
        formula: "H2SO4",
        composition: &[(1, 2), (8, 4), (16, 1)],
    },
    CompoundDefinition {
        name: "Sodium chloride",
        formula: "NaCl",
        composition: &[(11, 1), (17, 1)],
    },
    CompoundDefinition {
        name: "Calcium carbonate",
        formula: "CaCO3",
        composition: &[(6, 1), (8, 3), (20, 1)],
    },
    CompoundDefinition {
        name: "Silicon dioxide",
        formula: "SiO2",
        composition: &[(8, 2), (14, 1)],
    },
    CompoundDefinition {
        name: "Potassium nitrate",
        formula: "KNO3",
        composition: &[(7, 1), (8, 3), (19, 1)],
    },
    CompoundDefinition {
        name: "Methanol",
        formula: "CH3OH",
        composition: &[(1, 4), (6, 1), (8, 1)],
    },
    CompoundDefinition {
        name: "Ethane",
        formula: "C2H6",
        composition: &[(1, 6), (6, 2)],
    },
    CompoundDefinition {
        name: "Ethylene",
        formula: "C2H4",
        composition: &[(1, 4), (6, 2)],
    },
    CompoundDefinition {
        name: "Glucose",
        formula: "C6H12O6",
        composition: &[(1, 12), (6, 6), (8, 6)],
    },
    CompoundDefinition {
        name: "Iron(III) oxide",
        formula: "Fe2O3",
        composition: &[(8, 3), (26, 2)],
    },
    CompoundDefinition {
        name: "Aluminium oxide",
        formula: "Al2O3",
        composition: &[(8, 3), (13, 2)],
    },
    CompoundDefinition {
        name: "Calcium phosphate",
        formula: "Ca3(PO4)2",
        composition: &[(8, 8), (15, 2), (20, 3)],
    },
    CompoundDefinition {
        name: "Calcium hydroxide",
        formula: "Ca(OH)2",
        composition: &[(1, 2), (8, 2), (20, 1)],
    },
    CompoundDefinition {
        name: "Ammonia",
        formula: "NH3",
        composition: &[(1, 3), (7, 1)],
    },
    CompoundDefinition {
        name: "Methane",
        formula: "CH4",
        composition: &[(1, 4), (6, 1)],
    },
    CompoundDefinition {
        name: "Carbon dioxide",
        formula: "CO2",
        composition: &[(6, 1), (8, 2)],
    },
];

pub fn identify_compound(composition: &[(u8, u16)]) -> Option<&'static CompoundDefinition> {
    let normalized = normalize_composition(composition)?;
    KNOWN_COMPOUNDS
        .iter()
        .find(|compound| compound.composition == normalized.as_slice())
}

pub fn empirical_formula(composition: &[(u8, u16)]) -> Option<String> {
    let normalized = normalize_composition(composition)?;
    let mut formula = String::new();
    for (atomic_number, count) in normalized {
        let element = crate::elements::element_by_atomic_number(atomic_number)?;
        formula.push_str(element.symbol);
        if count > 1 {
            formula.push_str(&count.to_string());
        }
    }
    Some(formula)
}

fn normalize_composition(composition: &[(u8, u16)]) -> Option<Vec<(u8, u16)>> {
    let mut counts = BTreeMap::<u8, u16>::new();
    for (atomic_number, count) in composition {
        if *count == 0 || crate::elements::element_by_atomic_number(*atomic_number).is_none() {
            return None;
        }
        let entry = counts.entry(*atomic_number).or_default();
        *entry = entry.checked_add(*count)?;
    }
    if counts.is_empty() {
        return None;
    }
    Some(counts.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::{empirical_formula, identify_compound};

    #[test]
    fn compound_identification_is_order_independent() {
        let water = identify_compound(&[(8, 1), (1, 1), (1, 1)]).unwrap();
        let salt = identify_compound(&[(17, 1), (11, 1)]).unwrap();

        assert_eq!(water.formula, "H2O");
        assert_eq!(salt.name, "Sodium chloride");
    }

    #[test]
    fn compound_library_includes_documented_organic_and_mineral_examples() {
        assert_eq!(
            identify_compound(&[(6, 6), (1, 12), (8, 6)]).unwrap().name,
            "Glucose"
        );
        assert_eq!(
            identify_compound(&[(20, 3), (15, 2), (8, 8)])
                .unwrap()
                .formula,
            "Ca3(PO4)2"
        );
    }

    #[test]
    fn unknown_valid_compositions_keep_a_deterministic_formula() {
        assert_eq!(empirical_formula(&[(8, 1), (1, 2)]).as_deref(), Some("H2O"));
        assert!(identify_compound(&[(6, 1), (8, 1)]).is_none());
        assert_eq!(empirical_formula(&[(1, 0)]), None);
    }
}
