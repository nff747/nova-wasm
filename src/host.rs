//! Host environment imported function dispatcher.

use crate::trap::Trap;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type HostFn = Arc<dyn Fn(&[Value]) -> Result<Vec<Value>, Trap> + Send + Sync>;

#[derive(Default, Clone)]
pub struct HostRegistry {
    functions: HashMap<(String, String), HostFn>,
}

impl HostRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&mut self, module: &str, name: &str, f: F)
    where
        F: Fn(&[Value]) -> Result<Vec<Value>, Trap> + Send + Sync + 'static,
    {
        self.functions.insert((module.to_string(), name.to_string()), Arc::new(f));
    }

    pub fn call(&self, module: &str, name: &str, args: &[Value]) -> Result<Vec<Value>, Trap> {
        let f = self.functions.get(&(module.to_string(), name.to_string()))
            .ok_or(Trap::UndefinedElement)?;
        f(args)
    }
}
