use std::time;

use crate::interpreter::{Result, Value};

pub fn clock(_: Vec<Value>) -> Result<Value> {
    let secs = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    Ok(Value::Number(secs))
}
