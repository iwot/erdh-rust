extern crate postgres;
use postgres::{Connection, TlsMode};
use super::super::erdh::erdh_data::{Construction, Table, Column, Index, ForeignKey, get_relations_from_foreign_keys};
use super::super::config::db_config::DbConfig;
use std::collections::HashMap;

pub fn read_db(config: &DbConfig) -> Construction {
    // let constr = "postgres://user:pass@host:port/database";
    let user = config.user.clone().unwrap();
    let password = config.password.clone().unwrap();
    let host = config.host.clone().unwrap();
    let port = config.port.clone().unwrap_or("5432".to_string());
    let dbname = config.dbname.clone().unwrap();
    let constr = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, dbname);

    let conn = Connection::connect(constr, TlsMode::None).unwrap();

    let db_name = get_db_name(&conn).unwrap();

    let table_names = collect_table_names(&conn);

    let mut tables = Vec::new();
    for tbl in table_names {
        let columns = collect_table_columns(&conn, &db_name, &tbl);
        let indexes = collect_indexes(&conn, &db_name, &tbl);
        let foreign_keys = collect_foregin_keys(&conn, &db_name, &tbl);
        // foreign_keys から ex_relations を生成
        let ex_relations = get_relations_from_foreign_keys(&foreign_keys);
        tables.push(Table {
            table: tbl,
            group: db_name.clone(),
            columns: columns,
            indexes : indexes,
            foreign_keys : foreign_keys,
            ex_relations: ex_relations,
            is_master: None,
        });
    }

    Construction {
        db_name: db_name,
        tables: tables,
    }
}

pub fn get_db_name(conn: &Connection) -> Option<String> {
    let query = "SELECT current_database() AS db_name";

    let mut result = None;
    for row in &conn.query(query, &[]).unwrap() {
        result = Some(row.get("db_name"));
        break;
    }

    result
}

pub fn collect_table_names(conn: &Connection) -> Vec<String> {
    let query = r#"
    SELECT relname as table_name
      FROM pg_stat_user_tables"#;
    let mut result = vec![];
    for row in &conn.query(query, &[]).unwrap() {
        result.push(row.get("table_name"));
    }

    result
}

fn collect_primary_keys(conn: &Connection, db_name: &String, table_name: &String) ->HashMap<String, (String, String)> {
    let query = r#"
SELECT A.constraint_name
     , A.table_name
     , A.column_name
  FROM information_schema.key_column_usage A
       LEFT JOIN information_schema.table_constraints B ON A.constraint_name = B.constraint_name
 WHERE B.constraint_type = 'PRIMARY KEY'
   AND A.constraint_catalog = $1
   AND B.table_name = $2"#;

    let mut result = HashMap::new();

    for row in &conn.query(query, &[&db_name, &table_name]).unwrap() {
        let constraint_name: String = row.get("constraint_name");
        let table_name: String = row.get("table_name");
        let column_name: String = row.get("column_name");
        if !result.contains_key(&column_name) {
            result.insert(column_name, (constraint_name, table_name));
        }
    }

    result
}

pub fn collect_table_columns(conn: &Connection, db_name: &String, table_name: &String) -> Vec<Column> {
    let primary_key_checker = collect_primary_keys(&conn, &db_name, &table_name);

    let query = r#"
    SELECT column_name
     , data_type
     , character_maximum_length
     , numeric_precision
     , udt_name
     , column_default
     , is_nullable
  FROM information_schema.columns 
 WHERE table_catalog = $1
   AND table_name = $2
ORDER BY ordinal_position"#;
    let mut result = vec![];
    for row in &conn.query(query, &[&db_name, &table_name]).unwrap() {
        let is_nullable: String = row.get("is_nullable");
        let column_name: String = row.get("column_name");
        result.push(
            Column {
                name: column_name.clone(),
                column_type: row.get("data_type"),
                key: "".to_string(),
                extra: "".to_string(),
                default: row.get("column_default"),
                not_null: if is_nullable.to_lowercase() == "no" { true } else { false },
                is_primary: if primary_key_checker.contains_key(&column_name) { true } else { false },
            }
        );
    }

    result
}

pub fn collect_indexes(conn: &Connection, _db_name: &String, table_name: &String) -> Vec<Index> {
    let query = r"
        SELECT tablename
             , indexname
          FROM pg_indexes
         WHERE tablename = $1";
    let mut result = vec![];
    for row in &conn.query(query, &[&table_name]).unwrap() {
        result.push(
            Index {
                name: row.get("indexname"),
                column_name: "".to_string(),
            }
        );
    }

    result
}

pub fn collect_foregin_keys(conn: &Connection, _db_name: &String, table_name: &String) -> Vec<ForeignKey> {
    let query = r"
        SELECT tc.table_schema, 
               tc.constraint_name, 
               tc.table_name, 
               kcu.column_name, 
               ccu.table_schema AS referenced_column_schema,
               ccu.table_name AS referenced_table_name,
               ccu.column_name AS referenced_column_name 
          FROM information_schema.table_constraints AS tc 
               JOIN information_schema.key_column_usage AS kcu
                 ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
               JOIN information_schema.constraint_column_usage AS ccu
                 ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
         WHERE tc.constraint_type = 'FOREIGN KEY'
           AND tc.table_name = $1";
    let mut result = vec![];
    for row in &conn.query(query, &[&table_name]).unwrap() {
        result.push(
            ForeignKey {
                constraint_name: row.get("constraint_name"),
                column_name: row.get("column_name"),
                referenced_table_name: row.get("referenced_table_name"),
                referenced_column_name: row.get("referenced_column_name"),
            }
        );
    }

    result
}
