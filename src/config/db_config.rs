
use serde_derive::{Serialize, Deserialize};
extern crate serde_yaml;
use std::fs::File;
use std::io::prelude::*;
extern crate failure;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DbConfig {
    pub dbtype: DbType,
    pub host: Option<String>,
    pub port: Option<String>,
    pub dbname: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl DbConfig {
    pub fn from_yaml_file(path: &str) -> Result<DbConfig, failure::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_yaml(&contents)
    }

    pub fn from_yaml(yaml: &str) -> Result<DbConfig, failure::Error> {
        let result: DbConfig = serde_yaml::from_str(&yaml)?;
        Ok(result)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DbType {
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "postgres")]
    PostgreSQL,
    #[serde(rename = "sqlite")]
    SQLite,
}

#[test]
fn parse_yaml_for_mysql_data_success() {
    let yaml = r#"
dbtype: mysql
host: localhost
dbname: ELTEST01
user: root
password: password
    "#;
    let c = DbConfig::from_yaml(&yaml);
    assert_eq!(c.is_ok(), true);
}

#[test]
fn parse_yaml_for_postgres_data_success() {
    let yaml = r#"
dbtype: postgres
host: localhost
dbname: testdb
user: dev
password: password
    "#;
    let c = DbConfig::from_yaml(&yaml);
    assert_eq!(c.is_ok(), true);
}

#[test]
fn parse_yaml_for_sqlite_data_success() {
    let yaml = r#"
dbtype: sqlite
dbname: D:\works\github\erdh-clj-doc\test.sqlite3
    "#;
    let c = DbConfig::from_yaml(&yaml);
    assert_eq!(c.is_ok(), true);
}
