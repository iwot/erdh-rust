use serde_derive::{Serialize, Deserialize};
extern crate serde_yaml;
use std::fs::File;
use std::io::prelude::*;
extern crate failure;
use super::super::erdh::erdh_data;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtraConfig {
    pub tables: Vec<Table>,
}

impl ExtraConfig {
    pub fn from_yaml_file(path: &str) -> Result<ExtraConfig, failure::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_yaml(&contents)
    }

    pub fn from_yaml(yaml: &str) -> Result<ExtraConfig, failure::Error> {
        let result: ExtraConfig = serde_yaml::from_str(&yaml)?;
        Ok(result)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub table: String,
    pub is_master: Option<bool>,
    pub group: Option<String>,
    pub relations: Option<Vec<erdh_data::ExRelation>>,
}

#[test]
fn parse_yaml_data_success() {
    let yaml = r#"
tables:
- table: member_items
  is_master: true
  group: DATA
  relations:
    - referenced_table_name: members
      columns:
        - from: "member_id"
          to: "id"
      this_conn: "one"
      that_conn: "zero-or-one"
    - referenced_table_name: items
      columns:
        - from: "item_id"
          to: "id"
      this_conn: "onlyone"
      that_conn: "many"
- table: items
  is_master: true
  group: DATA
- table: item_types
  is_master: true
  group: MASTER
- table: member_items
  group: DATA
- table: members
  group: DATA
    "#;
    let c = ExtraConfig::from_yaml(&yaml);
    assert_eq!(c.is_ok(), true);
}
