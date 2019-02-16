use serde_derive::{Serialize, Deserialize};
extern crate serde_yaml;
use std::fs::File;
use std::io::prelude::*;
extern crate failure;

pub mod db_config;
pub mod extra_config;

// #[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub source: SourceType,
    pub source_from: String,
    pub group: Option<Vec<String>>,
    pub intermediate: Option<Intermediate>,
    pub ex_info: Option<String>,
}

impl Config {
    pub fn from_yaml_file(path: &str) -> Result<Config, failure::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_yaml(&contents)
    }

    pub fn from_yaml(yaml: &str) -> Result<Config, failure::Error> {
        let result: Config = serde_yaml::from_str(&yaml)?;
        Ok(result)
    }
}

// #[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Intermediate {
    pub save_to: Option<String>
}

// #[serde(rename_all = "lowercase")] // renameの代わりに使用すれば、すべてを小文字にして出力となる。
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "postgres")]
    PostgreSQL,
    #[serde(rename = "sqlite")]
    SQLite,
    #[serde(rename = "yaml")]
    YAML,
}

#[test]
fn parse_yaml_data_success() {
    let yaml = r#"
source: mysql
source_from: "D:\\works\\github\\erdh-clj-doc\\db_con_mysql.yaml"
group:
- DATA
- MASTER
intermediate:
  save_to: "D:\\works\\github\\erdh-clj-doc\\db_intermediate_m.yaml"
ex-info: "D:\\works\\github\\erdh-clj-doc\\ex_table_info.yaml"
    "#;
    let c = Config::from_yaml(&yaml);
    assert_eq!(c.is_ok(), true);
}
