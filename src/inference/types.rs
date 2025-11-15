//! Type system for stack effect inference

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stack type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StackType {
    Int,
    Float,
    Bool,
    Char,
    Addr,
    Unknown,
    Var(String),
}

impl fmt::Display for StackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackType::Int => write!(f, "n"),
            StackType::Float => write!(f, "f"),
            StackType::Bool => write!(f, "b"),
            StackType::Char => write!(f, "c"),
            StackType::Addr => write!(f, "a"),
            StackType::Unknown => write!(f, "x"),
            StackType::Var(name) => write!(f, "{}", name),
        }
    }
}

/// Stack effect (inputs -- outputs)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackEffect {
    pub inputs: Vec<StackType>,
    pub outputs: Vec<StackType>,
}

impl StackEffect {
    pub fn new(inputs: Vec<StackType>, outputs: Vec<StackType>) -> Self {
        Self { inputs, outputs }
    }

    /// Identity effect (no change)
    pub fn identity() -> Self {
        Self::new(vec![], vec![])
    }

    /// Stack depth change
    pub fn depth_delta(&self) -> i32 {
        self.outputs.len() as i32 - self.inputs.len() as i32
    }

    /// Compose two stack effects
    pub fn compose(&self, other: &StackEffect) -> Result<StackEffect, String> {
        // If we don't have enough outputs to satisfy other's inputs,
        // those inputs must come from our caller
        let shortfall = if other.inputs.len() > self.outputs.len() {
            other.inputs.len() - self.outputs.len()
        } else {
            0
        };

        // Build the new inputs: our inputs plus any shortfall
        let mut inputs = self.inputs.clone();
        if shortfall > 0 {
            // Add the needed inputs from the beginning of other's inputs
            for i in 0..shortfall {
                inputs.push(other.inputs[i].clone());
            }
        }

        // Build the new outputs
        let consumed_from_self = self.outputs.len().min(other.inputs.len());
        let remaining_outputs = self.outputs.len() - consumed_from_self;
        let mut outputs: Vec<StackType> = self.outputs[..remaining_outputs].to_vec();
        outputs.extend(other.outputs.clone());

        Ok(StackEffect::new(inputs, outputs))
    }

    /// Check if this effect is compatible with another
    pub fn compatible_with(&self, other: &StackEffect) -> bool {
        self.inputs.len() == other.inputs.len()
            && self.outputs.len() == other.outputs.len()
    }
}

impl fmt::Display for StackEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "( ")?;
        for (i, input) in self.inputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", input)?;
        }
        write!(f, " -- ")?;
        for (i, output) in self.outputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", output)?;
        }
        write!(f, " )")
    }
}

/// Information about an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationInfo {
    pub name: String,
    pub effect: StackEffect,
}

impl OperationInfo {
    pub fn new(name: impl Into<String>, effect: StackEffect) -> Self {
        Self {
            name: name.into(),
            effect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_delta() {
        let effect = StackEffect::new(
            vec![StackType::Int],
            vec![StackType::Int, StackType::Int],
        );
        assert_eq!(effect.depth_delta(), 1);
    }

    #[test]
    fn test_compose() {
        let dup = StackEffect::new(
            vec![StackType::Int],
            vec![StackType::Int, StackType::Int],
        );
        let add = StackEffect::new(
            vec![StackType::Int, StackType::Int],
            vec![StackType::Int],
        );

        let composed = dup.compose(&add).unwrap();
        assert_eq!(composed.inputs.len(), 1);
        assert_eq!(composed.outputs.len(), 1);
    }

    #[test]
    fn test_display() {
        let effect = StackEffect::new(
            vec![StackType::Int, StackType::Int],
            vec![StackType::Int],
        );
        assert_eq!(format!("{}", effect), "( n n -- n )");
    }
}
