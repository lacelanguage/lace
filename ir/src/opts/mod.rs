pub mod dce;
pub mod constant_fold;

use crate::core::module::Module;

pub trait Optimization {
    fn new() -> Self where Self: Sized;
    fn apply(&mut self, ir: &mut Module);
}

pub struct OptimizationHandler;

impl OptimizationHandler {
    pub fn get_opts(level: u32) -> Vec<Box<dyn Optimization>> {
        match level {
            1 => vec![Box::new(dce::DeadCodeElimination::new())],
            2 => vec![
                Box::new(constant_fold::ConstantFolder::new()),
                //Box::new(dce::DeadCodeElimination::new())
            ],
            _ => vec![]
        }
    }
}
