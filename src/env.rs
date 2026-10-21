use std::collections::HashMap;

use crate::interpreter::Value;

#[derive(Default, Clone)]
pub struct Env(HashMap<String, Value>);

impl Env {
    pub fn contains(&self, name: String) -> bool {
        self.0.contains_key(&name)
    }

    pub fn get(&self, name: String) -> Option<&Value> {
        self.0.get(&name)
    }

    pub fn set(&mut self, name: String, value: Value) -> Option<Value> {
        self.0.insert(name, value)
    }

    pub fn clone_with(&self, binds: Vec<(String, Value)>) -> Self {
        let mut env = self.clone();
        for b in binds {
            env.set(b.0, b.1);
        }
        env
    }
}
