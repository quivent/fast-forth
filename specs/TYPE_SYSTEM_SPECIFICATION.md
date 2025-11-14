# Fast Forth Type System Specification
**Version**: 1.0
**Date**: 2025-11-14

## Overview

Fast Forth uses a **Hindley-Milner based type system** adapted for stack-based computation. It provides static type safety while supporting Forth's dynamic, interactive development style through polymorphism and type inference.

---

## 1. Type System Foundations

### 1.1 Core Type Language

```rust
/// Type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Primitive types
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Char,

    /// Compound types
    String,
    Array(Box<Type>),
    Struct(StructId),
    Union(UnionId),
    Pointer(Box<Type>),

    /// Function type (stack effect)
    Function(StackEffect),

    /// Type variable (polymorphism)
    Var(TypeVar),

    /// Constrained type variable
    Constrained(TypeVar, Vec<Constraint>),

    /// Unknown type (during inference)
    Unknown(TypeVar),
}

/// Type variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

/// Stack effect signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackEffect {
    pub inputs: Vec<Type>,
    pub outputs: Vec<Type>,
}

impl StackEffect {
    /// Stack effect notation: ( inputs -- outputs )
    pub fn notation(&self) -> String {
        let inputs = self.inputs.iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let outputs = self.outputs.iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        format!("( {} -- {} )", inputs, outputs)
    }

    /// Compose two stack effects
    pub fn compose(&self, other: &StackEffect) -> Result<StackEffect, TypeError> {
        // Check output of self matches input of other
        if self.outputs.len() < other.inputs.len() {
            return Err(TypeError::StackUnderflow);
        }

        let consumed = self.outputs.len() - other.inputs.len();
        let mut new_outputs = self.outputs[..consumed].to_vec();

        // Type check the connection
        for (i, (out_ty, in_ty)) in self.outputs[consumed..]
            .iter()
            .zip(&other.inputs)
            .enumerate()
        {
            if !self.types_compatible(out_ty, in_ty) {
                return Err(TypeError::StackTypeMismatch {
                    position: i,
                    expected: in_ty.clone(),
                    found: out_ty.clone(),
                });
            }
        }

        new_outputs.extend(other.outputs.iter().cloned());

        Ok(StackEffect {
            inputs: self.inputs.clone(),
            outputs: new_outputs,
        })
    }
}

/// Type scheme for polymorphic definitions
#[derive(Debug, Clone)]
pub struct TypeScheme {
    /// Universally quantified type variables
    pub quantified: Vec<TypeVar>,

    /// Type constraints
    pub constraints: Vec<Constraint>,

    /// The actual type (with variables)
    pub ty: Type,
}

impl TypeScheme {
    /// Instantiate scheme with fresh type variables
    pub fn instantiate(&self, ctx: &mut TypeContext) -> Type {
        let substitution: HashMap<TypeVar, Type> = self.quantified
            .iter()
            .map(|&var| (var, Type::Unknown(ctx.fresh_var())))
            .collect();

        self.ty.apply_substitution(&substitution)
    }

    /// Generalize type to scheme by quantifying free variables
    pub fn generalize(ty: Type, env: &TypeEnvironment) -> TypeScheme {
        let free_in_env = env.free_type_vars();
        let free_in_ty = ty.free_type_vars();

        let quantified = free_in_ty
            .difference(&free_in_env)
            .cloned()
            .collect();

        TypeScheme {
            quantified,
            constraints: Vec::new(),
            ty,
        }
    }
}
```

### 1.2 Type Constraints

```rust
/// Constraints for type inference
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Equality constraint
    Equal(Type, Type),

    /// Type must be numeric
    Numeric(TypeVar),

    /// Type must be integral
    Integral(TypeVar),

    /// Type must be ordered (comparable)
    Ordered(TypeVar),

    /// Type must support operation
    SupportsOp(TypeVar, Operation),

    /// Type must have member
    HasMember(TypeVar, String, Type),

    /// Type must implement trait
    Implements(TypeVar, TraitId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Add, Sub, Mul, Div, Mod,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    ShiftLeft, ShiftRight,
}

/// Constraint solver
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    substitution: Substitution,
}

impl ConstraintSolver {
    pub fn solve(constraints: Vec<Constraint>) -> Result<Substitution, TypeError> {
        let mut solver = Self {
            constraints,
            substitution: Substitution::empty(),
        };

        while let Some(constraint) = solver.constraints.pop() {
            solver.solve_constraint(constraint)?;
        }

        Ok(solver.substitution)
    }

    fn solve_constraint(&mut self, constraint: Constraint) -> Result<(), TypeError> {
        match constraint {
            Constraint::Equal(t1, t2) => {
                let t1 = t1.apply_substitution(&self.substitution);
                let t2 = t2.apply_substitution(&self.substitution);
                let sub = self.unify(t1, t2)?;
                self.substitution = sub.compose(&self.substitution);
            }

            Constraint::Numeric(var) => {
                // Constrain to numeric types
                let concrete = self.substitution.get(var);
                if let Some(ty) = concrete {
                    if !matches!(ty, Type::Int32 | Type::Int64 | Type::Float32 | Type::Float64) {
                        return Err(TypeError::NotNumeric(ty.clone()));
                    }
                }
                // Otherwise, leave as constraint (will be resolved later)
            }

            Constraint::Integral(var) => {
                let concrete = self.substitution.get(var);
                if let Some(ty) = concrete {
                    if !matches!(ty, Type::Int32 | Type::Int64) {
                        return Err(TypeError::NotIntegral(ty.clone()));
                    }
                }
            }

            // ... other constraints
        }

        Ok(())
    }

    /// Robinson's unification algorithm
    fn unify(&self, t1: Type, t2: Type) -> Result<Substitution, TypeError> {
        match (t1, t2) {
            // Identical types
            (t1, t2) if t1 == t2 => Ok(Substitution::empty()),

            // Type variable unification
            (Type::Unknown(var), t) | (t, Type::Unknown(var)) => {
                if t.contains_var(var) {
                    return Err(TypeError::OccursCheck(var, t));
                }
                Ok(Substitution::singleton(var, t))
            }

            // Structural unification
            (Type::Array(t1), Type::Array(t2)) => {
                self.unify(*t1, *t2)
            }

            (Type::Pointer(t1), Type::Pointer(t2)) => {
                self.unify(*t1, *t2)
            }

            (Type::Function(e1), Type::Function(e2)) => {
                self.unify_stack_effects(e1, e2)
            }

            // Type mismatch
            (t1, t2) => Err(TypeError::CannotUnify(t1, t2)),
        }
    }

    fn unify_stack_effects(
        &self,
        e1: StackEffect,
        e2: StackEffect,
    ) -> Result<Substitution, TypeError> {
        if e1.inputs.len() != e2.inputs.len() || e1.outputs.len() != e2.outputs.len() {
            return Err(TypeError::StackEffectMismatch);
        }

        let mut sub = Substitution::empty();

        for (t1, t2) in e1.inputs.iter().zip(&e2.inputs) {
            let t1 = t1.apply_substitution(&sub);
            let t2 = t2.apply_substitution(&sub);
            let new_sub = self.unify(t1, t2)?;
            sub = new_sub.compose(&sub);
        }

        for (t1, t2) in e1.outputs.iter().zip(&e2.outputs) {
            let t1 = t1.apply_substitution(&sub);
            let t2 = t2.apply_substitution(&sub);
            let new_sub = self.unify(t1, t2)?;
            sub = new_sub.compose(&sub);
        }

        Ok(sub)
    }
}

/// Substitution (mapping from type variables to types)
#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<TypeVar, Type>,
}

impl Substitution {
    pub fn empty() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn singleton(var: TypeVar, ty: Type) -> Self {
        let mut map = HashMap::new();
        map.insert(var, ty);
        Self { map }
    }

    pub fn get(&self, var: TypeVar) -> Option<&Type> {
        self.map.get(&var)
    }

    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut map = self.map.clone();

        for (var, ty) in &other.map {
            let ty = ty.apply_substitution(self);
            map.insert(*var, ty);
        }

        Substitution { map }
    }
}

impl Type {
    pub fn apply_substitution(&self, sub: &Substitution) -> Type {
        match self {
            Type::Unknown(var) => {
                sub.get(*var).cloned().unwrap_or_else(|| self.clone())
            }

            Type::Array(elem) => {
                Type::Array(Box::new(elem.apply_substitution(sub)))
            }

            Type::Pointer(pointee) => {
                Type::Pointer(Box::new(pointee.apply_substitution(sub)))
            }

            Type::Function(effect) => {
                Type::Function(StackEffect {
                    inputs: effect.inputs.iter().map(|t| t.apply_substitution(sub)).collect(),
                    outputs: effect.outputs.iter().map(|t| t.apply_substitution(sub)).collect(),
                })
            }

            _ => self.clone(),
        }
    }

    pub fn free_type_vars(&self) -> HashSet<TypeVar> {
        match self {
            Type::Unknown(var) => {
                let mut set = HashSet::new();
                set.insert(*var);
                set
            }

            Type::Array(elem) | Type::Pointer(elem) => {
                elem.free_type_vars()
            }

            Type::Function(effect) => {
                let mut vars = HashSet::new();
                for ty in &effect.inputs {
                    vars.extend(ty.free_type_vars());
                }
                for ty in &effect.outputs {
                    vars.extend(ty.free_type_vars());
                }
                vars
            }

            _ => HashSet::new(),
        }
    }

    pub fn contains_var(&self, var: TypeVar) -> bool {
        self.free_type_vars().contains(&var)
    }
}
```

---

## 2. Type Inference Engine

### 2.1 Type Environment

```rust
/// Type environment (context)
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Word definitions
    words: HashMap<String, TypeScheme>,

    /// Local variables
    locals: HashMap<String, Type>,

    /// Parent environment (for scoping)
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            words: HashMap::new(),
            locals: HashMap::new(),
            parent: None,
        }
    }

    pub fn extend(&self) -> Self {
        Self {
            words: HashMap::new(),
            locals: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    pub fn define_word(&mut self, name: String, scheme: TypeScheme) {
        self.words.insert(name, scheme);
    }

    pub fn lookup_word(&self, name: &str) -> Option<&TypeScheme> {
        self.words.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup_word(name))
        })
    }

    pub fn free_type_vars(&self) -> HashSet<TypeVar> {
        let mut vars = HashSet::new();

        for scheme in self.words.values() {
            vars.extend(scheme.ty.free_type_vars());
        }

        for ty in self.locals.values() {
            vars.extend(ty.free_type_vars());
        }

        if let Some(parent) = &self.parent {
            vars.extend(parent.free_type_vars());
        }

        vars
    }
}
```

### 2.2 Type Inference Algorithm

```rust
/// Type inference context
pub struct TypeInferenceContext {
    /// Type environment
    env: TypeEnvironment,

    /// Fresh variable counter
    var_counter: u32,

    /// Accumulated constraints
    constraints: Vec<Constraint>,

    /// Stack type simulation
    stack_types: Vec<Type>,
}

impl TypeInferenceContext {
    pub fn new(env: TypeEnvironment) -> Self {
        Self {
            env,
            var_counter: 0,
            constraints: Vec::new(),
            stack_types: Vec::new(),
        }
    }

    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar(self.var_counter);
        self.var_counter += 1;
        var
    }

    /// Infer type for a definition
    pub fn infer_definition(&mut self, def: &ASTNode) -> Result<TypeScheme, TypeError> {
        match def {
            ASTNode::Definition { name, body, declared_effect } => {
                // Initialize stack with input types
                if let Some(effect) = declared_effect {
                    self.stack_types = effect.inputs.clone();
                } else {
                    // Create fresh variables for unknown inputs
                    self.stack_types = Vec::new();
                }

                // Infer body
                for stmt in body {
                    self.infer_statement(stmt)?;
                }

                // Create stack effect from inference
                let effect = StackEffect {
                    inputs: declared_effect.as_ref()
                        .map(|e| e.inputs.clone())
                        .unwrap_or_else(Vec::new),
                    outputs: self.stack_types.clone(),
                };

                // Solve constraints
                let substitution = ConstraintSolver::solve(self.constraints.clone())?;

                // Apply substitution to get concrete type
                let concrete_effect = effect.apply_substitution(&substitution);

                // Generalize to type scheme
                let scheme = TypeScheme::generalize(
                    Type::Function(concrete_effect),
                    &self.env
                );

                Ok(scheme)
            }

            _ => Err(TypeError::NotADefinition),
        }
    }

    /// Infer type for a statement
    fn infer_statement(&mut self, stmt: &ASTNode) -> Result<(), TypeError> {
        match stmt {
            ASTNode::Literal(lit) => {
                let ty = self.literal_type(lit);
                self.stack_types.push(ty);
                Ok(())
            }

            ASTNode::WordCall(name) => {
                // Lookup word
                let scheme = self.env.lookup_word(name)
                    .ok_or_else(|| TypeError::UndefinedWord(name.clone()))?;

                // Instantiate polymorphic type
                let ty = scheme.instantiate(self);

                // Extract stack effect
                if let Type::Function(effect) = ty {
                    // Check stack has enough elements
                    if self.stack_types.len() < effect.inputs.len() {
                        return Err(TypeError::StackUnderflow);
                    }

                    // Pop inputs and generate constraints
                    let start = self.stack_types.len() - effect.inputs.len();
                    let actual_inputs = self.stack_types.split_off(start);

                    for (expected, actual) in effect.inputs.iter().zip(actual_inputs) {
                        self.constraints.push(Constraint::Equal(
                            expected.clone(),
                            actual,
                        ));
                    }

                    // Push outputs
                    self.stack_types.extend(effect.outputs);

                    Ok(())
                } else {
                    Err(TypeError::NotAFunction(name.clone()))
                }
            }

            ASTNode::StackOp(op) => {
                self.infer_stack_op(*op)?;
                Ok(())
            }

            ASTNode::If { condition, then_branch, else_branch } => {
                // Condition must be boolean
                let cond_ty = self.stack_types.pop()
                    .ok_or(TypeError::StackUnderflow)?;

                self.constraints.push(Constraint::Equal(cond_ty, Type::Bool));

                // Infer both branches
                let stack_before = self.stack_types.clone();

                // Then branch
                let mut then_ctx = self.clone();
                for stmt in then_branch {
                    then_ctx.infer_statement(stmt)?;
                }
                let then_stack = then_ctx.stack_types.clone();

                // Else branch
                self.stack_types = stack_before;
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.infer_statement(stmt)?;
                    }
                }
                let else_stack = self.stack_types.clone();

                // Both branches must have same stack effect
                if then_stack.len() != else_stack.len() {
                    return Err(TypeError::BranchStackMismatch);
                }

                for (then_ty, else_ty) in then_stack.iter().zip(&else_stack) {
                    self.constraints.push(Constraint::Equal(
                        then_ty.clone(),
                        else_ty.clone(),
                    ));
                }

                self.stack_types = then_stack;
                Ok(())
            }

            _ => todo!("Infer other statement types"),
        }
    }

    fn infer_stack_op(&mut self, op: StackOp) -> Result<(), TypeError> {
        match op {
            StackOp::Dup => {
                let ty = self.stack_types.last()
                    .ok_or(TypeError::StackUnderflow)?
                    .clone();
                self.stack_types.push(ty);
                Ok(())
            }

            StackOp::Drop => {
                self.stack_types.pop()
                    .ok_or(TypeError::StackUnderflow)?;
                Ok(())
            }

            StackOp::Swap => {
                if self.stack_types.len() < 2 {
                    return Err(TypeError::StackUnderflow);
                }
                let len = self.stack_types.len();
                self.stack_types.swap(len - 1, len - 2);
                Ok(())
            }

            StackOp::Over => {
                if self.stack_types.len() < 2 {
                    return Err(TypeError::StackUnderflow);
                }
                let ty = self.stack_types[self.stack_types.len() - 2].clone();
                self.stack_types.push(ty);
                Ok(())
            }

            StackOp::Rot => {
                if self.stack_types.len() < 3 {
                    return Err(TypeError::StackUnderflow);
                }
                let len = self.stack_types.len();
                let a = self.stack_types.remove(len - 3);
                self.stack_types.push(a);
                Ok(())
            }

            _ => todo!("Other stack ops"),
        }
    }

    fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int32(_) => Type::Int32,
            Literal::Int64(_) => Type::Int64,
            Literal::Float32(_) => Type::Float32,
            Literal::Float64(_) => Type::Float64,
            Literal::Bool(_) => Type::Bool,
            Literal::Char(_) => Type::Char,
            Literal::String(_) => Type::String,
        }
    }
}
```

---

## 3. Type Inference Examples

### 3.1 Simple Function

```forth
: SQUARE ( n -- n² )
  DUP * ;
```

**Inference Steps**:

1. Initial stack effect: `( α -- ? )`
2. DUP: `( α -- α α )`
3. `*`: Requires numeric operands
   - Constraint: `Numeric(α)`
   - Effect: `( num num -- num )`
4. Result: `( num -- num )`

**Type Scheme**:
```
∀α. Numeric(α) ⇒ ( α -- α )
```

### 3.2 Polymorphic Function

```forth
: SWAP-DUP ( a b -- b a a )
  SWAP DUP ;
```

**Inference Steps**:

1. Initial: `( α β -- ? )`
2. SWAP: `( α β -- β α )`
3. DUP: `( β α -- β α α )`
4. Result: `( α β -- β α α )`

**Type Scheme**:
```
∀α β. ( α β -- β α α )
```

### 3.3 Constrained Polymorphism

```forth
: MIN ( a b -- min )
  2DUP < IF DROP ELSE NIP THEN ;
```

**Inference Steps**:

1. Initial: `( α β -- ? )`
2. 2DUP: `( α β -- α β α β )`
3. `<`: Requires comparable types
   - Constraint: `Ordered(α)`, `Equal(α, β)`
   - Effect: `( α β α β -- α β bool )`
4. IF: Consumes boolean
   - Then: DROP → `( α β -- α )`
   - Else: NIP → `( α β -- β )`
   - Both must unify: `Equal(α, β)`
5. Result: `( α α -- α )`

**Type Scheme**:
```
∀α. Ordered(α) ⇒ ( α α -- α )
```

### 3.4 Higher-Order Function

```forth
: MAP ( addr len xt -- )  \ Apply xt to each element
  -ROT  \ xt addr len
  0 DO
    2DUP I CELLS + @ SWAP  \ addr xt value xt
    EXECUTE                 \ addr xt result
    SWAP 2DUP I CELLS + !   \ Store result
    SWAP
  LOOP
  2DROP ;
```

**Type Scheme**:
```
∀α β. ( Array(α) (α -- β) -- Array(β) )
```

---

## 4. Primitive Type Signatures

```rust
// Built-in word type signatures
pub fn builtin_types() -> HashMap<String, TypeScheme> {
    let mut types = HashMap::new();

    // Arithmetic (polymorphic over numeric types)
    types.insert("+".to_string(), TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![Constraint::Numeric(TypeVar(0))],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Unknown(TypeVar(0)), Type::Unknown(TypeVar(0))],
            outputs: vec![Type::Unknown(TypeVar(0))],
        }),
    });

    types.insert("*".to_string(), TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![Constraint::Numeric(TypeVar(0))],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Unknown(TypeVar(0)), Type::Unknown(TypeVar(0))],
            outputs: vec![Type::Unknown(TypeVar(0))],
        }),
    });

    // Comparison (polymorphic over ordered types)
    types.insert("<".to_string(), TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![Constraint::Ordered(TypeVar(0))],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Unknown(TypeVar(0)), Type::Unknown(TypeVar(0))],
            outputs: vec![Type::Bool],
        }),
    });

    // Stack operations (fully polymorphic)
    types.insert("DUP".to_string(), TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Unknown(TypeVar(0))],
            outputs: vec![Type::Unknown(TypeVar(0)), Type::Unknown(TypeVar(0))],
        }),
    });

    types.insert("SWAP".to_string(), TypeScheme {
        quantified: vec![TypeVar(0), TypeVar(1)],
        constraints: vec![],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Unknown(TypeVar(0)), Type::Unknown(TypeVar(1))],
            outputs: vec![Type::Unknown(TypeVar(1)), Type::Unknown(TypeVar(0))],
        }),
    });

    // Memory operations
    types.insert("@".to_string(), TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Pointer(Box::new(Type::Unknown(TypeVar(0))))],
            outputs: vec![Type::Unknown(TypeVar(0))],
        }),
    });

    types.insert("!".to_string(), TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![],
        ty: Type::Function(StackEffect {
            inputs: vec![
                Type::Unknown(TypeVar(0)),
                Type::Pointer(Box::new(Type::Unknown(TypeVar(0)))),
            ],
            outputs: vec![],
        }),
    });

    types
}
```

---

## 5. Type Error Reporting

```rust
#[derive(Debug, Clone)]
pub enum TypeError {
    UndefinedWord(String),
    NotAFunction(String),
    StackUnderflow,
    StackEffectMismatch,
    CannotUnify(Type, Type),
    OccursCheck(TypeVar, Type),
    NotNumeric(Type),
    NotIntegral(Type),
    NotOrdered(Type),
    BranchStackMismatch,
    StackTypeMismatch {
        position: usize,
        expected: Type,
        found: Type,
    },
}

impl TypeError {
    pub fn format(&self, source: &str, location: &SourceLocation) -> String {
        let context = self.extract_context(source, location);

        match self {
            TypeError::StackUnderflow => {
                format!(
                    "Type error at {}:{}\n  Stack underflow: not enough values on stack\n{}",
                    location.line, location.column, context
                )
            }

            TypeError::CannotUnify(t1, t2) => {
                format!(
                    "Type error at {}:{}\n  Cannot unify types:\n    {}\n  with:\n    {}\n{}",
                    location.line, location.column, t1, t2, context
                )
            }

            TypeError::StackTypeMismatch { position, expected, found } => {
                format!(
                    "Type error at {}:{}\n  Stack position {}: expected {}, found {}\n{}",
                    location.line, location.column, position, expected, found, context
                )
            }

            _ => format!("{:?}", self),
        }
    }

    fn extract_context(&self, source: &str, location: &SourceLocation) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = location.line as usize - 1;

        if line_idx >= lines.len() {
            return String::new();
        }

        let line = lines[line_idx];
        let col = location.column as usize;

        format!(
            "\n  {}\n  {}^\n",
            line,
            " ".repeat(col)
        )
    }
}
```

---

## 6. Type System Configuration

```rust
pub struct TypeSystemConfig {
    /// Strict mode: require stack effect annotations
    pub strict_effects: bool,

    /// Allow implicit type conversions
    pub allow_implicit_conversions: bool,

    /// Maximum type inference iterations
    pub max_inference_iterations: usize,

    /// Enable type specialization
    pub enable_specialization: bool,
}

impl Default for TypeSystemConfig {
    fn default() -> Self {
        Self {
            strict_effects: true,
            allow_implicit_conversions: false,
            max_inference_iterations: 100,
            enable_specialization: true,
        }
    }
}
```

---

This type system provides strong static guarantees while maintaining Forth's flexibility through polymorphism and type inference. All development streams working on type checking should reference this specification.
