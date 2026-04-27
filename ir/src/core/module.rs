use std::fmt;
use lasso::{Rodeo, Spur};
use super::function::{Function, FunctionName, Signature};

pub struct Module {
    pub id: usize,
    pub name: String,
    pub functions: Vec<Function>
}

impl Module {
    pub fn new<S: AsRef<str>>(id: usize, name: S) -> Self {
        Self {
            id,
            name: name.as_ref().to_string(),
            functions: vec![]
        }
    }

    pub fn define_function(&mut self, scope: usize, name: Spur, sig: Signature) -> FunctionName {
        self.functions.push(Function::new(name, self.functions.len(), self.id, scope, sig));
        self.functions.last().unwrap().name
    }

    pub fn get_function(&mut self, name: &FunctionName) -> Option<&mut Function> {
        self.functions.iter_mut().find(|f| f.name == *name)
    }

    pub fn debug(&self, rodeo: &Rodeo) -> String {
        let mut output = String::new();

        output.push_str(&format!("@module_name({})\n@module_export(\n\t", self.name));

        for (idx, func) in self.functions.iter().enumerate() {
            if idx > 0 {
                output.push_str(",\n\t");
            }
            output.push_str(&func.debug_sig());
        }
        output.push_str("\n)");

        for func in &self.functions {
            output.push_str("\n\n");
            output.push_str(&func.debug(rodeo));
        }

        output
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@module_name({})\n@module_export(", self.name)?;

        for (idx, func) in self.functions.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", func.debug_sig())?;
        }
        write!(f, ")")?;

        for func in &self.functions {
            write!(f, "\n\n{func:?}")?;
        }

        Ok(())
    }
}