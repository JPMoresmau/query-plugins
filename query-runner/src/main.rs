use std::path::Path;

use anyhow::{anyhow, Result};
use query::Query;
use wasmer::*;
use wasmer_compiler_llvm::LLVM;

wai_bindgen_wasmer::import!("../test-collect/query.wai");

/// Greet using all the plugins.
fn main() -> Result<()> {
    let compiler_config = LLVM::default();
    let engine = EngineBuilder::new(compiler_config).engine();

    let path = Path::new("../test-collect/target/wasm32-unknown-unknown/release/test_collect.wasm");
    let mut store = Store::new(&engine);

    let module = Module::from_file(&store, &path)?;

    let mut imports = imports! {};
    let (query, _instance) = Query::instantiate(&mut store, &module, &mut imports)?;

    let metadata = query.metadata(&mut store)?;
    println!("Metadata: {metadata:?}");

    let vars = vec![crate::query::VariableParam {
        name: "customer_id",
        value: crate::query::ValueParam::DataInteger(123),
    }];
    let execution = query.start(&mut store, &vars)?;
    println!(
        "Query: {}",
        query.execution_query_string(&mut store, &execution)?
    );

    query.execution_row(
        &mut store,
        &execution,
        &[crate::query::VariableParam {
            name: "order_id",
            value: crate::query::ValueParam::DataInteger(1234),
        }],
    )?;
    query.execution_row(
        &mut store,
        &execution,
        &[crate::query::VariableParam {
            name: "order_id",
            value: crate::query::ValueParam::DataInteger(1235),
        }],
    )?;

    let result = query.execution_end(&mut store, &execution)?;
    println!("Result: {result:?}");
    Ok(())
}
