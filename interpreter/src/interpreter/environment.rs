use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::Value;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Default, Debug)]
pub struct Environment {
    pub enclosing: Option<EnvRef>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn child(enclosing: EnvRef) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) {
            return Some(v.clone());
        }

        match &self.enclosing {
            Some(parent) => parent.borrow().get(name),
            None => None,
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Option<()> {
        if let Some(v) = self.values.get_mut(name) {
            *v = value.clone();
            return Some(());
        }

        match &mut self.enclosing {
            Some(parent) => parent.borrow_mut().assign(name, value),
            None => None,
        }
    }
}
