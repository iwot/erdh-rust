use serde_derive::{Serialize, Deserialize};
extern crate serde_yaml;
use std::fs::File;
use std::io::prelude::*;
extern crate failure;
use std::collections::HashMap;


#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Construction {
    pub db_name: String,
    pub tables: Vec<Table>,
}

impl Construction {
    pub fn from_yaml_file(path: &str) -> Result<Construction, failure::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_yaml(&contents)
    }

    pub fn from_yaml(yaml: &str) -> Result<Construction, failure::Error> {
        let result: Construction = serde_yaml::from_str(&yaml)?;
        Ok(result)
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub table: String,
    pub group: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub ex_relations: Vec<ExRelation>,
    pub is_master: Option<bool>,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub key: String,
    pub extra: String,
    pub default: Option<String>,
    pub not_null: bool,
    pub is_primary: bool,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub column_name: String,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ForeignKey {
    pub constraint_name: String,
    pub column_name: String,
    pub referenced_table_name: String,
    pub referenced_column_name: String,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExRelation {
    pub referenced_table_name: String,
    pub columns: Vec<ExRelationColumn>,
    pub this_conn: Connection,
    pub that_conn: Connection,
}

impl ExRelation {
    pub fn get_clone(&self) -> ExRelation {
        ExRelation {
            referenced_table_name: self.referenced_table_name.clone(),
            columns: self.columns.iter().map(|r| r.get_clone()).collect(),
            this_conn: self.this_conn.clone(),
            that_conn: self.that_conn.clone(),
        }
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExRelationColumn {
    pub from: String,
    pub to: String,
}

impl ExRelationColumn {
    fn get_clone(&self) -> ExRelationColumn {
        ExRelationColumn {
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Connection {
    #[serde(rename = "one")]
    One,
    #[serde(rename = "onlyone")]
    OnlyOne,
    #[serde(rename = "zero-or-one")]
    ZeroOrOne,
    #[serde(rename = "many")]
    Many,
    #[serde(rename = "one-more")]
    OneMore,
    #[serde(rename = "zero-many")]
    ZeroMany,
}

pub fn get_relations_from_foreign_keys(foreign_keys: &Vec<ForeignKey>) -> Vec<ExRelation> {
    let mut collect = HashMap::new();
    for fk in foreign_keys {
        let rel = collect.entry(fk.referenced_table_name.clone())
                         .or_insert_with(|| ExRelation { 
                                            referenced_table_name: fk.referenced_table_name.clone(),
                                            columns: vec![],
                                            this_conn: Connection::One,
                                            that_conn: Connection::One,
                         });
        rel.columns.push(ExRelationColumn {
            from: fk.column_name.clone(),
            to: fk.referenced_column_name.clone(),
            });
    }

    let mut result = vec![];
    for (_, v) in collect {
        result.push(v);
    }
    result
}

#[test]
fn parse_yaml_data_success() {
    let yaml = r#"
---
db_name: test.sqlite3
tables:
  - table: members
    group: DATA
    columns:
      - name: id
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: false
        is_primary: false
      - name: name
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: true
        is_primary: false
      - name: gender
        type: INT(32)
        key: ""
        extra: ""
        default: "0"
        not_null: true
        is_primary: false
    indexes: []
    foreign_keys: []
    ex_relations: []
    is_master: ~
  - table: member_items
    group: DATA
    columns:
      - name: id
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: false
        is_primary: false
      - name: member_id
        type: INT(32)
        key: ""
        extra: ""
        default: ~
        not_null: true
        is_primary: false
      - name: enable
        type: INT(32)
        key: ""
        extra: ""
        default: "0"
        not_null: true
        is_primary: false
      - name: item_id
        type: INT(32)
        key: ""
        extra: ""
        default: ~
        not_null: true
        is_primary: false
      - name: num
        type: INT(32)
        key: ""
        extra: ""
        default: "0"
        not_null: true
        is_primary: false
    indexes: []
    foreign_keys: []
    ex_relations:
      - referenced_table_name: members
        columns:
          - from: member_id
            to: id
        this_conn: one
        that_conn: zero-or-one
      - referenced_table_name: items
        columns:
          - from: item_id
            to: id
        this_conn: onlyone
        that_conn: many
    is_master: true
  - table: item_types
    group: MASTER
    columns:
      - name: id
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: false
        is_primary: false
      - name: name
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: true
        is_primary: false
    indexes: []
    foreign_keys: []
    ex_relations: []
    is_master: true
  - table: items
    group: DATA
    columns:
      - name: id
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: false
        is_primary: false
      - name: name
        type: TEXT
        key: ""
        extra: ""
        default: ~
        not_null: true
        is_primary: false
      - name: type
        type: INT(32)
        key: ""
        extra: ""
        default: ~
        not_null: true
        is_primary: false
    indexes: []
    foreign_keys: []
    ex_relations: []
    is_master: true
    "#;
    let c = Construction::from_yaml(&yaml);
    assert_eq!(c.is_ok(), true);
}
