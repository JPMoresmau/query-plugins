use std::collections::HashMap;

use anyhow::{anyhow, Result};
use query_runner::{
    query::ParameterType, DBConnection, QueryResult, State, ValueParam, ValueResult,
};

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
    integer_result("memory")
}

fn integer_result(connection: &str) -> Result<()> {
    let st = test_state()?;

    if let DBConnection::SqliteConnection(conn) = st.get_connection(connection)? {
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
    }

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st
        .run_untyped("test_collect", connection, &variables)?
        .unwrap();
    assert_result(
        &res,
        &["order_id"],
        &[
            &[ValueParam::DataInteger(Some(1234))],
            &[ValueParam::DataInteger(Some(1235))],
        ],
    );

    Ok(())
}

#[test]
fn sqlite_text() -> Result<()> {
    result_text("memory")
}

fn result_text(connection: &str) -> Result<()> {
    let st = test_state()?;

    if let DBConnection::SqliteConnection(conn) = st.get_connection(connection)? {
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
    }

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st
        .run_untyped("test_collect", connection, &variables)?
        .unwrap();
    assert_result(
        &res,
        &["order_id"],
        &[
            &[ValueParam::DataString(Some("1234"))],
            &[ValueParam::DataString(Some("1235"))],
        ],
    );

    Ok(())
}

#[test]
fn sqlite_bool() -> Result<()> {
    result_bool("memory")
}

fn result_bool(connection: &str) -> Result<()> {
    let st = test_state()?;

    if let DBConnection::SqliteConnection(conn) = st.get_connection(connection)? {
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
    }

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st
        .run_untyped("test_collect", connection, &variables)?
        .unwrap();
    assert_result(
        &res,
        &["order_id"],
        &[
            &[ValueParam::DataBoolean(Some(false))],
            &[ValueParam::DataBoolean(Some(true))],
        ],
    );
    Ok(())
}

#[test]
fn sqlite_decimal() -> Result<()> {
    decimal_result("memory")
}

fn decimal_result(connection: &str) -> Result<()> {
    let st = test_state()?;

    if let DBConnection::SqliteConnection(conn) = st.get_connection(connection)? {
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
    }

    let mut variables = HashMap::new();
    variables.insert("customer_id", "123");
    let res = st
        .run_untyped("test_collect", connection, &variables)?
        .unwrap();
    assert_result(
        &res,
        &["order_id"],
        &[
            &[ValueParam::DataDecimal(Some(123.4))],
            &[ValueParam::DataDecimal(Some(123.5))],
        ],
    );

    Ok(())
}

#[test]
fn sqlite_null_result() -> Result<()> {
    null_result("memory")
}

#[test]
fn postgres_null_result() -> Result<()> {
    null_result("postgres1")
}

fn null_result(connection: &str) -> Result<()> {
    let st = test_state()?;

    if let DBConnection::SqliteConnection(conn) = st.get_connection(connection)? {
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
    }
    let mut variables = HashMap::new();
    variables.insert("user_name", "john");
    let res = st
        .run_untyped("test_collect2", connection, &variables)?
        .unwrap();
    assert_result(
        &res,
        &["name", "email"],
        &[&[
            ValueParam::DataString(Some("John Doe")),
            ValueParam::DataString(Some("john.doe@example.com")),
        ]],
    );

    let mut variables = HashMap::new();
    variables.insert("user_name", "jane");
    let res = st
        .run_untyped("test_collect2", connection, &variables)?
        .unwrap();
    assert_result(
        &res,
        &["name", "email"],
        &[&[
            ValueParam::DataString(Some("Jane Doe")),
            ValueParam::DataString(None),
        ]],
    );

    Ok(())
}

fn test_state() -> Result<State> {
    State::load_from_disk()
}

fn assert_result(res: &QueryResult, names: &[&str], values: &[&[ValueParam]]) {
    assert_eq!(names.len(), res.names.len());
    for (expected, got) in names.iter().zip(res.names.iter()) {
        assert_eq!(expected, got)
    }
    assert_eq!(values.len(), res.values.len());
    for (expected, got) in values.iter().zip(res.values.iter()) {
        assert_eq!(expected.len(), got.len());
        for (expected_value, got_value) in expected.iter().zip(got.iter()) {
            match expected_value {
                ValueParam::DataBoolean(b1) => {
                    assert!(matches!(got_value, ValueResult::DataBoolean(b2) if b1 == b2))
                }
                ValueParam::DataDecimal(d1) => {
                    assert!(matches!(got_value, ValueResult::DataDecimal(d2) if d1 == d2))
                }
                ValueParam::DataInteger(i1) => {
                    assert!(matches!(got_value, ValueResult::DataInteger(i2) if i1 == i2))
                }
                ValueParam::DataString(s1) => assert!(
                    matches!(got_value, ValueResult::DataString(s2) if s1 == &s2.as_deref())
                ),
                ValueParam::DataTimestamp(t1) => assert!(
                    matches!(got_value, ValueResult::DataTimestamp(t2) if t1 == &t2.as_deref())
                ),
            }
        }
    }
}
