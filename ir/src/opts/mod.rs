pub mod dce;

use crate::core::module::Module;

pub trait Optimization {
    fn new() -> Self where Self: Sized;
    fn apply(&mut self, ir: &mut Module);
}

pub struct OptimizationHandler;

impl OptimizationHandler {
    pub fn get_opts(level: u32) -> Vec<impl Optimization> {
        let mut opts = vec![];

        if level >= 1 {
            opts.push(dce::DeadCodeElimination::new());
        }
        
        opts
    }
}
