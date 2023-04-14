use std::cell::RefCell;

use wai_bindgen_rust::Handle;

wai_bindgen_rust::export!("query.wai");

use crate::query::*;

use paste::paste;
use query_common::*;

pub struct Query {}

impl crate::query::Query for Query {
    fn metadata() -> QueryMetadata {
        metadata!("test plugins collecting results",
        "customer_id" => Integer)
    }

    fn start(variables: Vec<Variable>) -> Handle<Execution> {
        Execution {
            query_string: String::from(
                "SELECT order_id FROM Orders WHERE customer_id = {{customer_id}}",
            ),
            variables,
            data: RefCell::from(Vec::new()),
        }
        .into()
    }
}

pub struct Execution {
    query_string: String,
    variables: Vec<Variable>,
    data: RefCell<Vec<Vec<Variable>>>,
}

impl crate::query::Execution for Execution {
    fn query_string(&self) -> String {
        self.query_string.clone()
    }

    fn variables(&self) -> Vec<Variable> {
        self.variables.clone()
    }

    fn row(&self, data: Vec<Variable>) -> Option<Vec<Vec<Variable>>> {
        self.data.borrow_mut().push(data);
        None
    }

    fn end(&self) -> Option<Vec<Vec<Variable>>> {
        Some(self.data.borrow_mut().drain(..).collect())
    }
}
