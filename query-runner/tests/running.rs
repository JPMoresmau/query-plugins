use std::collections::HashMap;

use anyhow::{anyhow, Result};
use query_runner::{query::ParameterType, DBConnection, State};

#[test]
fn ok_module() -> Result<()> {
    let st = test_state()?;
    assert!(st.get_plugin("test_collect").is_ok());
    Ok(())
}

#[test]
fn err_module() -> Result<()> {
    let st = test_state()?;
    assert!(st.get_plugin("test_collect2").is_err());
    Ok(())
}

#[test]
fn module_parameters() -> Result<()> {
    let st = test_state()?;
    let m = st.get_plugin("test_collect")?;
    let params = st.get_parameters(m)?;
    assert_eq!(1, params.len());
    let p = params.get(0).ok_or(anyhow!("no parameter at index 0"))?;
    assert_eq!("customer_id", &p.name);
    assert_eq!(ParameterType::TypeInteger, p.parameter_type);
    Ok(())
}

#[test]
fn sqlite_integer() -> Result<()> {
    let st = test_state()?;

    let DBConnection::SqliteConnection(conn) = st.get_connection("memory")?;
    conn.execute(
        "CREATE TABLE Orders (
            order_id     INTEGER PRIMARY KEY,
            customer_id  INTEGER NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "INSERT INTO Orders (order_id, customer_id) VALUES (?1, ?2), (?3, ?4)",
        (1234, 123, 1235, 123),
    )?;

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st.run_untyped("test_collect", "memory", &variables)?;
    let mut res = res.unwrap();
    assert_eq!(1, res.names.len());
    assert_eq!("order_id", res.names[0]);
    assert_eq!(2, res.values.len());
    let mut v = Vec::new();
    while let Some(mut row) = res.values.pop() {
        assert_eq!(1, row.len());
        let vr = row.pop().unwrap();
        match vr {
            query_runner::ValueResult::DataInteger(i) => v.push(i),
            _ => panic!("unxpected result {:?}", vr),
        }
    }
    assert_eq!(2, v.len());
    assert!(v.contains(&1234));
    assert!(v.contains(&1235));
    Ok(())
}

#[test]
fn sqlite_text() -> Result<()> {
    let st = test_state()?;

    let DBConnection::SqliteConnection(conn) = st.get_connection("memory")?;
    conn.execute(
        "CREATE TABLE Orders (
            order_id     TEXT PRIMARY KEY,
            customer_id  INTEGER NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "INSERT INTO Orders (order_id, customer_id) VALUES (?1, ?2), (?3, ?4)",
        ("1234", 123, "1235", 123),
    )?;

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st.run_untyped("test_collect", "memory", &variables)?;
    let mut res = res.unwrap();
    assert_eq!(1, res.names.len());
    assert_eq!("order_id", res.names[0]);
    assert_eq!(2, res.values.len());
    let mut v = Vec::new();
    while let Some(mut row) = res.values.pop() {
        assert_eq!(1, row.len());
        let vr = row.pop().unwrap();
        match vr {
            query_runner::ValueResult::DataString(s) => v.push(s),
            _ => panic!("unxpected result {:?}", vr),
        }
    }
    assert_eq!(2, v.len());
    // <https://github.com/rust-lang/rust/issues/42671>
    assert!(v.contains(&String::from("1234")));
    assert!(v.contains(&String::from("1235")));
    Ok(())
}

#[test]
fn sqlite_bool() -> Result<()> {
    let st = test_state()?;

    let DBConnection::SqliteConnection(conn) = st.get_connection("memory")?;
    conn.execute(
        "CREATE TABLE Orders (
            order_id     BOOL,
            customer_id  INTEGER NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "INSERT INTO Orders (order_id, customer_id) VALUES (?1, ?2), (?3, ?4)",
        (true, 123, false, 123),
    )?;

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st.run_untyped("test_collect", "memory", &variables)?;
    let mut res = res.unwrap();
    assert_eq!(1, res.names.len());
    assert_eq!("order_id", res.names[0]);
    assert_eq!(2, res.values.len());
    let mut v = Vec::new();
    while let Some(mut row) = res.values.pop() {
        assert_eq!(1, row.len());
        let vr = row.pop().unwrap();
        match vr {
            query_runner::ValueResult::DataBoolean(s) => v.push(s),
            _ => panic!("unxpected result {:?}", vr),
        }
    }
    assert_eq!(2, v.len());
    assert!(v.contains(&true));
    assert!(v.contains(&false));
    Ok(())
}

#[test]
fn sqlite_decimal() -> Result<()> {
    let st = test_state()?;

    let DBConnection::SqliteConnection(conn) = st.get_connection("memory")?;
    conn.execute(
        "CREATE TABLE Orders (
            order_id     REAL,
            customer_id  INTEGER NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "INSERT INTO Orders (order_id, customer_id) VALUES (?1, ?2), (?3, ?4)",
        (123.4, 123, 123.5, 123),
    )?;

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st.run_untyped("test_collect", "memory", &variables)?;
    let mut res = res.unwrap();
    assert_eq!(1, res.names.len());
    assert_eq!("order_id", res.names[0]);
    assert_eq!(2, res.values.len());
    let mut v = Vec::new();

    while let Some(mut row) = res.values.pop() {
        assert_eq!(1, row.len());
        let vr = row.pop().unwrap();
        match vr {
            query_runner::ValueResult::DataDecimal(s) => v.push(s),
            _ => panic!("unxpected result {:?}", vr),
        }
    }
    assert_eq!(2, v.len());
    assert!(v.contains(&123.4));
    assert!(v.contains(&123.5));
    Ok(())
}

fn test_state() -> Result<State> {
    State::load_from_disk()
}
