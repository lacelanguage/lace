use std::fmt;
use lasso::{Rodeo, Spur};
use super::function::{Function, FunctionName, Signature};

pub struct Module {
    id: usize,
    name: String,
    functions: Vec<Function>
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
        self.functions.push(Function::new(name, self.id, scope, sig));
        self.functions.last().unwrap().name
    }

    pub fn get_function(&mut self, name: &FunctionName) -> Option<&mut Function> {
        self.functions.iter_mut().find(|f| f.name == *name)
    }

    pub fn debug(&self, rodeo: &Rodeo) -> String {
        let mut output = String::new();

        output.push_str(&format!("@module_name({})\n\n", self.name));

        for func in &self.functions {
            output.push_str(&func.debug(rodeo));
            output.push_str("\n\n");
        }

        output
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@module_name({})\n\n", self.name)?;

        for func in &self.functions {
            write!(f, "{func:?}\n\n")?;
        }

        Ok(())
    }
}