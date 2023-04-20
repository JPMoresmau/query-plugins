use std::collections::HashMap;

use anyhow::{anyhow, Result};
use query_runner::{query::ParameterType, DBConnection, State, ValueResult};

#[test]
fn ok_module() -> Result<()> {
    let st = test_state()?;
    assert!(st.get_plugin("test_collect").is_ok());
    Ok(())
}

#[test]
fn err_module() -> Result<()> {
    let st = test_state()?;
    assert!(st.get_plugin("test_collect_missing").is_err());
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
    assert!(v.contains(&Some(1234)));
    assert!(v.contains(&Some(1235)));
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
    assert!(v.contains(&Some(String::from("1234"))));
    assert!(v.contains(&Some(String::from("1235"))));
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
    assert!(v.contains(&Some(true)));
    assert!(v.contains(&Some(false)));
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
    assert!(v.contains(&Some(123.4)));
    assert!(v.contains(&Some(123.5)));
    Ok(())
}

#[test]
fn sqlite_null_result() -> Result<()> {
    let st = test_state()?;

    let DBConnection::SqliteConnection(conn) = st.get_connection("memory")?;
    conn.execute(
        "CREATE TABLE Users (
            username  TEXT PRIMARY KEY,
            name  TEXT NOT NULL,
            email TEXT
        )",
        (),
    )?;
    conn.execute(
        "INSERT INTO Users (username, name, email) VALUES (?1, ?2, ?3), (?4, ?5, NULL)",
        (
            "john",
            "John Doe",
            "john.doe@example.com",
            "jane",
            "Jane Doe",
        ),
    )?;
    let mut variables = HashMap::new();
    variables.insert("user_name", "john");
    let res = st.run_untyped("test_collect2", "memory", &variables)?;
    let res = res.unwrap();
    assert_eq!(2, res.names.len());
    assert_eq!("name", res.names[0]);
    assert_eq!("email", res.names[1]);
    assert_eq!(1, res.values.len());
    assert_eq!(2, res.values[0].len());
    assert!(
        matches!(&res.values[0][0], ValueResult::DataString(n) if n == &Some("John Doe".to_string()))
    );
    assert!(
        matches!(&res.values[0][1], ValueResult::DataString(n) if n == &Some("john.doe@example.com".to_string()))
    );

    let mut variables = HashMap::new();
    variables.insert("user_name", "jane");
    let res = st.run_untyped("test_collect2", "memory", &variables)?;
    let res = res.unwrap();
    assert_eq!(2, res.names.len());
    assert_eq!("name", res.names[0]);
    assert_eq!("email", res.names[1]);
    assert_eq!(1, res.values.len());
    assert_eq!(2, res.values[0].len());
    assert!(
        matches!(&res.values[0][0], ValueResult::DataString(n) if n == &Some("Jane Doe".to_string()))
    );
    assert!(matches!(&res.values[0][1], ValueResult::DataString(n) if n.is_none()));

    Ok(())
}

fn test_state() -> Result<State> {
    State::load_from_disk()
}
