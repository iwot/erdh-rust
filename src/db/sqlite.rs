extern crate sqlite3;
extern crate nom_sql;
extern crate regex;
use super::super::erdh::erdh_data::{Construction, Table, Column};
use super::super::config::db_config::DbConfig;
use regex::Regex;
use std::path::Path;

pub fn read_db(config: &DbConfig) -> Construction {
    let db_name = config.dbname.clone().unwrap();
    let conn = sqlite3::open(&db_name).unwrap();
    let table_create_data = read_table_creates(&conn);

    let path = Path::new(&db_name);
    let db_file_name = path.file_name().unwrap().to_str().unwrap();

    let mut tables = Vec::new();

    for (table_name, create_query) in &table_create_data {
        if let Some(table) = parse_create_query(&create_query, &db_file_name, &table_name) {
            tables.push(table);
        }
    }
    Construction {
        db_name: db_file_name.to_string(),
        tables: tables,
    }
}

/// Vec<(テーブル名, クリエイト文)>を返す。
pub fn read_table_creates(conn: &sqlite3::Connection) -> Vec<(String, String)> {
    let mut result = vec![];

    let query = r#"SELECT tbl_name, sql FROM sqlite_master WHERE type = "table""#;
    let mut cursor = conn.prepare(query).unwrap().cursor();

    while let Some(row) = cursor.next().unwrap() {
        result.push((row[0].as_string().unwrap().to_string(), row[1].as_string().unwrap().to_string()));
    }

    result
}

fn parse_create_query(create_query: &str, db_name: &str, table_name: &str) -> Option<Table> {
    // 簡易的にコメントを削除
    let re = Regex::new(r"--.+(\r\n|\n)").unwrap();
    let cq = re.replace_all(&create_query, " ");
    let result = match nom_sql::parser::parse_query(&cq) {
        Ok(nom_sql::parser::SqlQuery::CreateTable(q)) => {
            let mut columns = vec![];
            for spec in &q.fields {
                // spec.sql_type => Text || Int(32) ...
                let mut default_value = None;
                for c in &spec.constraints {
                    match c {
                        nom_sql::ColumnConstraint::DefaultValue(v) => {
                            default_value = Some(v.to_string());
                        },
                        _ => {},
                    };
                }
                columns.push(
                    Column {
                        name: spec.column.name.clone(),
                        column_type: format!("{}", spec.sql_type),
                        key: "".to_string(),
                        extra: "".to_string(),
                        default: default_value,
                        not_null: if spec.constraints.contains(&nom_sql::ColumnConstraint::NotNull) { true } else { false },
                        is_primary: false,
                    }
                );
            }

            let table = Table {
                table: table_name.to_string(),
                group: db_name.to_string(),
                columns: columns,
                indexes : vec![],
                foreign_keys : vec![],
                ex_relations: vec![],
                is_master: None,
            };

            Some(table)
        },
        Err(e) => {
            println!("create query parsing failed => {}", create_query);
            dbg!(e);

            None
        },
        _ => {
            println!("unexpected data found");

            None
        },
    };

    result
}
