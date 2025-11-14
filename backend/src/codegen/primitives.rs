//! Primitive Operation Code Generation
//!
//! Generate native code for Forth primitive operations (+, -, *, /, etc.)

use crate::error::{BackendError, Result};
use fastforth_frontend::ssa::{BinaryOperator, UnaryOperator};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::{BasicValueEnum, IntValue, FloatValue};
use inkwell::{IntPredicate, FloatPredicate};

/// Primitive operation code generator
pub struct PrimitiveCodegen<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> PrimitiveCodegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Generate code for binary operation
    pub fn generate_binary_op(
        &self,
        builder: &Builder<'ctx>,
        op: BinaryOperator,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        match op {
            // Arithmetic operations
            BinaryOperator::Add => self.gen_add(builder, lhs, rhs),
            BinaryOperator::Sub => self.gen_sub(builder, lhs, rhs),
            BinaryOperator::Mul => self.gen_mul(builder, lhs, rhs),
            BinaryOperator::Div => self.gen_div(builder, lhs, rhs),
            BinaryOperator::Mod => self.gen_mod(builder, lhs, rhs),

            // Comparison operations
            BinaryOperator::Lt => self.gen_lt(builder, lhs, rhs),
            BinaryOperator::Gt => self.gen_gt(builder, lhs, rhs),
            BinaryOperator::Le => self.gen_le(builder, lhs, rhs),
            BinaryOperator::Ge => self.gen_ge(builder, lhs, rhs),
            BinaryOperator::Eq => self.gen_eq(builder, lhs, rhs),
            BinaryOperator::Ne => self.gen_ne(builder, lhs, rhs),

            // Logical operations
            BinaryOperator::And => self.gen_and(builder, lhs, rhs),
            BinaryOperator::Or => self.gen_or(builder, lhs, rhs),
        }
    }

    /// Generate code for unary operation
    pub fn generate_unary_op(
        &self,
        builder: &Builder<'ctx>,
        op: UnaryOperator,
        operand: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        match op {
            UnaryOperator::Negate => self.gen_negate(builder, operand),
            UnaryOperator::Not => self.gen_not(builder, operand),
            UnaryOperator::Abs => self.gen_abs(builder, operand),
        }
    }

    // Arithmetic operations

    fn gen_add(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_add(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "add"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_add(
                lhs.into_float_value(),
                rhs.into_float_value(),
                "fadd"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in add operation".to_string()))
        }
    }

    fn gen_sub(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_sub(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "sub"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_sub(
                lhs.into_float_value(),
                rhs.into_float_value(),
                "fsub"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in sub operation".to_string()))
        }
    }

    fn gen_mul(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_mul(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "mul"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_mul(
                lhs.into_float_value(),
                rhs.into_float_value(),
                "fmul"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in mul operation".to_string()))
        }
    }

    fn gen_div(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_signed_div(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "div"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_div(
                lhs.into_float_value(),
                rhs.into_float_value(),
                "fdiv"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in div operation".to_string()))
        }
    }

    fn gen_mod(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_signed_rem(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "mod"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Mod operation requires integer operands".to_string()))
        }
    }

    // Comparison operations

    fn gen_lt(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_compare(
                IntPredicate::SLT,
                lhs.into_int_value(),
                rhs.into_int_value(),
                "lt"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            // Extend to i64 for Forth compatibility
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "lt_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_compare(
                FloatPredicate::OLT,
                lhs.into_float_value(),
                rhs.into_float_value(),
                "flt"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "flt_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in lt operation".to_string()))
        }
    }

    fn gen_gt(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_compare(
                IntPredicate::SGT,
                lhs.into_int_value(),
                rhs.into_int_value(),
                "gt"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "gt_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_compare(
                FloatPredicate::OGT,
                lhs.into_float_value(),
                rhs.into_float_value(),
                "fgt"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "fgt_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in gt operation".to_string()))
        }
    }

    fn gen_le(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_compare(
                IntPredicate::SLE,
                lhs.into_int_value(),
                rhs.into_int_value(),
                "le"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "le_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in le operation".to_string()))
        }
    }

    fn gen_ge(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_compare(
                IntPredicate::SGE,
                lhs.into_int_value(),
                rhs.into_int_value(),
                "ge"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "ge_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in ge operation".to_string()))
        }
    }

    fn gen_eq(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_compare(
                IntPredicate::EQ,
                lhs.into_int_value(),
                rhs.into_int_value(),
                "eq"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "eq_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else if lhs.is_float_value() && rhs.is_float_value() {
            let result = builder.build_float_compare(
                FloatPredicate::OEQ,
                lhs.into_float_value(),
                rhs.into_float_value(),
                "feq"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "feq_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in eq operation".to_string()))
        }
    }

    fn gen_ne(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_int_compare(
                IntPredicate::NE,
                lhs.into_int_value(),
                rhs.into_int_value(),
                "ne"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            let extended = builder.build_int_z_extend(
                result,
                self.context.i64_type(),
                "ne_ext"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(extended.into())
        } else {
            Err(BackendError::CodeGenError("Type mismatch in ne operation".to_string()))
        }
    }

    // Logical operations

    fn gen_and(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_and(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "and"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("And operation requires integer operands".to_string()))
        }
    }

    fn gen_or(
        &self,
        builder: &Builder<'ctx>,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if lhs.is_int_value() && rhs.is_int_value() {
            let result = builder.build_or(
                lhs.into_int_value(),
                rhs.into_int_value(),
                "or"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Or operation requires integer operands".to_string()))
        }
    }

    // Unary operations

    fn gen_negate(
        &self,
        builder: &Builder<'ctx>,
        operand: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if operand.is_int_value() {
            let result = builder.build_int_neg(
                operand.into_int_value(),
                "neg"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else if operand.is_float_value() {
            let result = builder.build_float_neg(
                operand.into_float_value(),
                "fneg"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Invalid operand type for negate".to_string()))
        }
    }

    fn gen_not(
        &self,
        builder: &Builder<'ctx>,
        operand: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if operand.is_int_value() {
            let result = builder.build_not(
                operand.into_int_value(),
                "not"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            Ok(result.into())
        } else {
            Err(BackendError::CodeGenError("Not operation requires integer operand".to_string()))
        }
    }

    fn gen_abs(
        &self,
        builder: &Builder<'ctx>,
        operand: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        if operand.is_int_value() {
            let val = operand.into_int_value();
            let zero = self.context.i64_type().const_zero();

            // abs(x) = x < 0 ? -x : x
            let is_negative = builder.build_int_compare(
                IntPredicate::SLT,
                val,
                zero,
                "is_neg"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;

            let negated = builder.build_int_neg(val, "neg")
                .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

            let result = builder.build_select(
                is_negative,
                negated,
                val,
                "abs"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;

            Ok(result)
        } else if operand.is_float_value() {
            // Use LLVM intrinsic for float abs
            let val = operand.into_float_value();
            let abs_fn = self.get_float_abs_intrinsic(builder);
            let result = builder.build_call(
                abs_fn,
                &[val.into()],
                "fabs"
            ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;

            Ok(result.try_as_basic_value().left().unwrap())
        } else {
            Err(BackendError::CodeGenError("Invalid operand type for abs".to_string()))
        }
    }

    /// Get LLVM intrinsic for floating-point abs
    fn get_float_abs_intrinsic(&self, builder: &Builder<'ctx>) -> inkwell::values::FunctionValue<'ctx> {
        let module = builder.get_insert_block().unwrap().get_parent().unwrap().get_parent().unwrap();
        let f64_type = self.context.f64_type();

        if let Some(func) = module.get_function("llvm.fabs.f64") {
            func
        } else {
            let fn_type = f64_type.fn_type(&[f64_type.into()], false);
            module.add_function("llvm.fabs.f64", fn_type, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_primitive_codegen_creation() {
        let context = Context::create();
        let primitives = PrimitiveCodegen::new(&context);
        assert_eq!(primitives.context.i64_type().get_bit_width(), 64);
    }
}
