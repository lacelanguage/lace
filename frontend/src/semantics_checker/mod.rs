pub mod ty;
pub mod symbol;
pub mod type_map;

const CANDIDATE_SCORE_THRESHOLD: f64 = 0.7;

use ty::Type;
use symbol::*;
use type_map::{TypeMap, FunctionDefTypeInfo};
use crate::parser::ast::{Ast, Node, NodeKind, RootLevelItem, RootLevelItemKind};
use crate::parser::ty::*;
use lace_span::Span;
use crate::operator::Op;
use crate::diagnostic::{Diagnostic, Severity};
use std::collections::HashMap;
use lasso::{Spur, Rodeo};

pub struct SemanticsChecker<'a> {
    pub type_registry: HashMap<Spur, Type>,
    pub scope: Vec<HashMap<Spur, Symbol>>,
    pub type_map: TypeMap,
    pub rodeo: &'a Rodeo,
}

impl<'a> SemanticsChecker<'a> {
    pub fn new(rodeo: &'a mut Rodeo) -> Self {
        let mut type_registry = HashMap::new();

        type_registry.insert(rodeo.get_or_intern("int"), Type::Int);
        type_registry.insert(rodeo.get_or_intern("float"), Type::Float);
        type_registry.insert(rodeo.get_or_intern("bool"), Type::Bool);

        Self {
            type_registry,
            scope: vec![HashMap::new()],
            type_map: TypeMap::new(),
            rodeo: &*rodeo
        }
    }

    pub fn define_symbol(&mut self, mutability: bool, name: Spur, ty: Type, defined_at: Span) {
        self.scope.last_mut()
            .unwrap()
            .insert(name, Symbol { mutability, ty, defined_at });
    }

    pub fn get_identifier(&self, spur: &Spur) -> (Option<&Symbol>, Option<(&str, &Symbol)>) {
        let mut candidate = None;
        let mut last_score = 0.0;

        for frame in self.scope.iter().rev() {
            for (sp, symbol) in frame {
                if spur == sp {
                    return (Some(symbol), None);
                } else {
                    let c = self.rodeo.resolve(sp);
                    let score = strsim::jaro_winkler(
                        c,
                        self.rodeo.resolve(spur)
                    );
                    if score > CANDIDATE_SCORE_THRESHOLD && score > last_score {
                        last_score = score;
                        candidate = Some((c, symbol));
                    }
                }
            }
        }

        (None, candidate)
    }

    pub fn get_identifier_mut(&mut self, spur: &Spur) -> (Option<&mut Symbol>, Option<(&str, &Symbol)>) {
        let mut candidate = None;
        let mut last_score = 0.0;

        for frame in self.scope.iter_mut().rev() {
            for (sp, symbol) in frame {
                if spur == sp {
                    return (Some(symbol), None);
                } else {
                    let c = self.rodeo.resolve(sp);
                    let score = strsim::jaro_winkler(
                        c,
                        self.rodeo.resolve(spur)
                    );
                    if score > CANDIDATE_SCORE_THRESHOLD && score > last_score {
                        last_score = score;
                        candidate = Some((c, &*symbol));
                    }
                }
            }
        }

        (None, candidate)
    }

    pub fn resolve_type(&self, ty: &ParseType) -> Result<Type, Diagnostic> {
        match &ty.kind {
            ParseTypeKind::Identifier(n) => self.type_registry.get(n)
                .ok_or(Diagnostic::new(
                    Severity::Error,
                    format!("type `{}` is not defined", self.rodeo.resolve(n)),
                    ty.span
                )).cloned()
        }
    }

    pub fn check(&mut self, ast: &Ast) -> Result<(), Vec<Diagnostic>> {
        // pass 1: collect
        for item in ast.0.iter() {
            self.collect_root_level_item(item)?;
        }

        // pass 2: check
        for item in ast.0.iter() {
            self.check_root_level_item(item)?;
        }

        Ok(())
    }

    pub fn collect_root_level_item(&mut self, item: &RootLevelItem) -> Result<(), Vec<Diagnostic>> {
        match &item.kind {
            RootLevelItemKind::FunctionDef(f) => {
                let mut errors = Vec::new();

                if let Some(symbol) = self.get_identifier(&f.name).0 {
                    return Err(vec![Diagnostic::new(
                        Severity::Error,
                        format!("function `{}` was already defined", self.rodeo.resolve(&f.name)),
                        item.span
                    ).with_note(format!("function `{}` was defined here:", self.rodeo.resolve(&f.name)), Some(symbol.defined_at))]);
                }

                let mut param_tys = Vec::new();
                for param in &f.params {
                    let result = self.resolve_type(&param.ty);
                    match result {
                        Ok(ty) => param_tys.push(ty),
                        Err(err) => errors.push(err),
                    }
                }
                let ret = if let Some(ty) = &f.return_ty {
                    match self.resolve_type(ty) {
                        Ok(ty) => Some(ty),
                        Err(err) => {
                            errors.push(err);
                            None
                        }
                    }
                } else {
                    Some(Type::Unit)
                };

                if !errors.is_empty() {
                    return Err(errors);
                }

                self.type_map.assign_func(
                    f.id,
                    FunctionDefTypeInfo {
                        params: param_tys.clone(),
                        return_ty: ret.clone().unwrap(),
                        defined_at: item.span
                });
                self.define_symbol(
                    false,
                    f.name,
                    Type::Function(param_tys, Box::new(ret.unwrap())),
                    item.span
                );
                Ok(())
            },
        }
    }

    pub fn check_root_level_item(&mut self, item: &RootLevelItem) -> Result<(), Vec<Diagnostic>> {
        match &item.kind {
            RootLevelItemKind::FunctionDef(f) => {
                self.scope.push(HashMap::new());
                let mut params = vec![];
                {
                    let FunctionDefTypeInfo { params: param_tys, .. } = self.type_map.get_func(f.id).unwrap();
                    for (idx, p) in param_tys.iter().enumerate() {
                        params.push((f.params[idx].mutability, f.params[idx].name, p.clone(), f.params[idx].span));
                    }
                }
                for (mutability, name, ty, defined_at) in params {
                    self.define_symbol(mutability, name, ty, defined_at);
                }
                self.check_node(&f.body)?;
                self.scope.pop();

                
                let FunctionDefTypeInfo { return_ty, defined_at, .. } = self.type_map.get_func(f.id).unwrap();
                let body_ty = self.type_map.get_node(f.body.id).unwrap();
                if *return_ty != *body_ty {
                    return Err(vec![Diagnostic::new(
                        Severity::Error,
                        format!("function defined with return type `{return_ty}` but returns `{body_ty}`"),
                        *defined_at
                    )]);
                }
            },
        };

        Ok(())
    }

    pub fn check_node(&mut self, node: &Node) -> Result<(), Vec<Diagnostic>> {
        let mut errors = Vec::new();

        match &node.kind {
            NodeKind::Identifier(s) => {
                let (symbol, candidate) = self.get_identifier(s);
                if let Some(s) = symbol {
                    let ty = s.ty.clone();
                    self.type_map.assign_node(node.id, ty.clone());
                    return Ok(());
                } else if let Some(c) = candidate {
                    errors.push(Diagnostic::new(
                        Severity::Error,
                        format!("`{}` not found in scope", self.rodeo.resolve(s)),
                        node.span
                    ).with_help(format!("did you mean `{}`?", c.0), None));
                } else {
                    errors.push(Diagnostic::new(
                        Severity::Error,
                        format!("`{}` not found in scope", self.rodeo.resolve(s)),
                        node.span
                    ));
                }
            },
            NodeKind::IntLit(_) => {
                self.type_map.assign_node(node.id, Type::Int);
                return Ok(());
            },
            NodeKind::FloatLit(_) => {
                self.type_map.assign_node(node.id, Type::Float);
                return Ok(());
            },
            NodeKind::Unit => {
                self.type_map.assign_node(node.id, Type::Unit);
                return Ok(());
            },
            NodeKind::Semi(stmt) => {
                self.check_node(stmt)?;
                self.type_map.assign_node(node.id, Type::Unit);
                return Ok(());
            },
            NodeKind::Tuple(items) => {
                let mut item_tys = Vec::new();

                for item in items {
                    match self.check_node(item) {
                        Ok(_) => item_tys.push(self.type_map.get_node(item.id).unwrap().clone()),
                        Err(err) => errors.extend(err),
                    }
                }

                if !errors.is_empty() {
                    return Err(errors);
                }

                self.type_map.assign_node(node.id, Type::Tuple(item_tys));
                return Ok(());
            },
            NodeKind::Block(stmts) => {
                let mut last_ty = None;

                self.scope.push(HashMap::new());

                for stmt in stmts {
                    match self.check_node(stmt) {
                        Ok(_) => last_ty = Some(self.type_map.get_node(stmt.id).unwrap().clone()),
                        Err(err) => errors.extend(err),
                    }
                }

                self.scope.pop();

                if !errors.is_empty() {
                    return Err(errors);
                }

                self.type_map.assign_node(node.id, last_ty.unwrap_or(Type::Unit));

                return Ok(());
            },
            NodeKind::BinaryOp { lhs, rhs, op } => {
                if op.0 == Op::Assign {
                    if let NodeKind::Identifier(s) = &lhs.kind {
                        let name = self.rodeo.resolve(s);
                        self.check_node(rhs)?;
                        let val_ty = self.type_map.get_node(rhs.id).unwrap().clone();
                        let (symbol, candidate) = self.get_identifier_mut(s);
                        if let Some(s) = symbol {
                            if s.ty != val_ty {
                                return Err(vec![Diagnostic::new(
                                    Severity::Error,
                                    format!("`{name}` is of type `{}` but found type `{val_ty}`", s.ty),
                                    node.span
                                ).with_note(format!("`{name}` was defined here:"), Some(s.defined_at))]);
                            } else if !s.mutability {
                                return Err(vec![Diagnostic::new(
                                    Severity::Error,
                                    format!("cannot mutate immutable variable `{name}`"),
                                    node.span
                                ).with_note(format!("`{name}` was defined here:"), Some(s.defined_at))]);
                            }

                            self.type_map.assign_node(node.id, Type::Unit);
                            return Ok(());
                        } else if let Some(c) = candidate {
                            errors.push(Diagnostic::new(
                                Severity::Error,
                                format!("`{name}` not found in scope"),
                                node.span
                            ).with_help(format!("did you mean `{}`?", c.0), None));
                        } else {
                            errors.push(Diagnostic::new(
                                Severity::Error,
                                format!("`{name}` not found in scope"),
                                node.span
                            ));
                        }
                    } else {
                        errors.push(Diagnostic::new(
                            Severity::Error,
                            "expected expression, found `=`",
                            op.1
                        ));
                    }

                    if !errors.is_empty() {
                        return Err(errors);
                    }
                }

                if let Err(err) = self.check_node(lhs) {
                    errors.extend(err);
                }
                if let Err(err) = self.check_node(rhs) {
                    errors.extend(err);
                }

                if !errors.is_empty() {
                    return Err(errors);
                }

                if let Some(ty) = op.0.infix_output_ty(
                    self.type_map.get_node(lhs.id).unwrap(),
                    self.type_map.get_node(rhs.id).unwrap()
                ) {
                    self.type_map.assign_node(node.id, ty);
                    return Ok(());
                } else {
                    errors.push(Diagnostic::new(
                        Severity::Error,
                        format!(
                            "cannot apply `{}` as a binary operation on types `{}` and `{}`",
                            op.0,
                            self.type_map.get_node(lhs.id).unwrap(),
                            self.type_map.get_node(rhs.id).unwrap()
                        ),
                        node.span
                    ));
                }
            },
            NodeKind::UnaryOp { operand, op } => {
                self.check_node(operand)?;
                if let Some(ty) = op.0.prefix_output_ty(
                    self.type_map.get_node(operand.id).unwrap()
                ) {
                    self.type_map.assign_node(node.id, ty);
                    return Ok(())
                } else {
                    errors.push(Diagnostic::new(
                        Severity::Error,
                        format!(
                            "cannot apply `{}` as a unary operation on type `{}`",
                            op.0,
                            self.type_map.get_node(operand.id).unwrap()
                        ),
                        node.span
                    ));
                }
            },
            NodeKind::Let { mutability, name, ty, value } => {
                self.check_node(value)?;
                let final_ty = match ty {
                    Some(ty) => {
                        let resolved_ty = self.resolve_type(ty).map_err(|err| vec![err])?;
                        let value_ty = self.type_map.get_node(value.id).unwrap().clone();
                        if resolved_ty != value_ty {
                            return Err(vec![Diagnostic::new(
                                Severity::Error,
                                format!("`{}` defined as `{resolved_ty}` but initialized as `{value_ty}`", self.rodeo.resolve(name)),
                                node.span
                            )]);
                        }

                        value_ty
                    },
                    None => {
                        self.type_map.get_node(value.id).unwrap().clone()
                    }
                };

                self.define_symbol(*mutability, *name, final_ty.clone(), node.span);
                self.type_map.assign_node(node.id, final_ty);
                return Ok(());
            },
            NodeKind::FunctionDef(_f) => todo!("scoped functions")
        }

        Err(errors)
    }
}