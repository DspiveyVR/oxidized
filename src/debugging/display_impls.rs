// I think I'll eventually want to make this a TUI

use color_print::cwrite;
use std::fmt::Display;

use crate::static_analysis::data_structures::*;

impl Display for ConcreteRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        cwrite!(f, "<yellow>'{} -> {}</>", self.name, self.loans)
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
            "{} {}",
            self.ownership_qualifier.to_string(),
            self.place_expression
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
            PlaceExpression::Variable(var) => write!(f, "{}", var),
        }
    }
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
