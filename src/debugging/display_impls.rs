// I think I'll eventually want to make this a TUI

use std::fmt::Display;

use crate::static_analysis::data_structures::*;

impl Display for TypeComputation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entries = self
            .clone()
            .into_iter()
            .map(|(typ, pe_pointer)| {
                let place_str =
                    if &pe_pointer.place_expression == &(pe_pointer.points_to.clone().into()) {
                        pe_pointer.points_to.to_string()
                    } else {
                        format!(
                            "({} -> {})",
                            pe_pointer.place_expression, pe_pointer.points_to
                        )
                    };
                format!("{place_str}: {typ}")
            })
            .collect::<Vec<String>>();

        write!(f, "[{}]", entries.join(", "))
    }
}

impl Display for ConcreteTyping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let regions_str = self
            .regions
            .0
            .iter()
            .fold("".to_string(), |a, e| format!("{},\n\t\t{}", a, e))
            .get(1..)
            .unwrap_or("")
            .to_string();
        let types_str = self
            .types
            .0
            .iter()
            .fold("".to_string(), |a, e| format!("{},\n\t\t{}", a, e))
            .get(1..)
            .unwrap_or("")
            .to_string();

        write!(
            f,
            "ConcreteTyping {{ \n\tr: [{}\n\t]\n \t𝜏: [{}\n\t]\n }}",
            regions_str, types_str
        )
    }
}

impl Display for ReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "&'{} {} {}",
            self.region_name,
            self.ownership_qualifier.to_string(),
            self.target_type
        )
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let binding_str = if let Some(binding) = &self.binding {
            format!(
                "{} {} := ",
                binding.is_mut.then(|| "mut").unwrap_or(""),
                binding.variable
            )
        } else {
            "".to_string()
        };
        let variant_str = match &self.variant {
            TypeVariant::Base(base_type) => match base_type {
                BaseType::Bool => "bool".to_string(),
                BaseType::U32 => "u32".to_string(),
                BaseType::Unit => "()".to_string(),
            },
            TypeVariant::Reference(reference_type) => format!(
                "&'{} {} {}",
                reference_type.region_name,
                reference_type.ownership_qualifier.to_string(),
                reference_type.target_type
            ),
            TypeVariant::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .fold("".to_string(), |a, e| format!("{}, {}", a, e))
                    .get(2..)
                    .unwrap_or("")
                    .to_string()
            ),
        };

        write!(
            f,
            "{}",
            if self.is_dead == Ternary::Yes {
                format!("({}{})†", binding_str, variant_str)
            } else {
                format!("{}{}", binding_str, variant_str)
            }
        )
    }
}

impl Display for ConcreteRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{} -> {}", self.name, self.loans)
    }
}

impl Display for VecSet<Loan> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {} }}",
            self.0
                .iter()
                .map(|l| format!("{}", l))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Display for Loan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.ownership_qualifier.to_string(),
            if self.is_reborrow { "*" } else { "" },
            self.place
        )
    }
}

impl Display for PlaceExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceExpression::Deref(place_expression) => write!(f, "*({})", place_expression),
            PlaceExpression::PathAccess(place_expression, path) => {
                write!(f, "({}){}", place_expression, path_to_string(path))
            }
            PlaceExpression::Place(place) => write!(f, "{}", place),
        }
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.variable,
            if self.path.len() > 0 {
                path_to_string(&self.path)
            } else {
                "".to_string()
            }
        )
    }
}

pub fn _vec_display_elements<T>(vec: &Vec<T>) -> String
where
    T: Display,
{
    format!(
        "[\n{}\n]",
        vec.iter()
            .fold("".to_string(), |a, e| format!("\t{},\n {}", a, e))
            .to_string()
    )
}

fn path_to_string(path: &Vec<u32>) -> String {
    format!(
        ".{}",
        path.iter()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
            .join(".")
    )
}

pub fn _format_explode_pair(pair: (Place, Type)) -> String {
    format!("{{{}, {}}}", pair.0, pair.1)
}
