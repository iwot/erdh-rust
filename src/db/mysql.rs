extern crate mysql;
extern crate r2d2_mysql;
extern crate r2d2;
use mysql as my;
use super::super::erdh::erdh_data::{Construction, Table, Column, Index, ForeignKey, get_relations_from_foreign_keys};
use super::super::config::db_config::DbConfig;

pub fn get_opts(config: &DbConfig) -> my::Opts {
    let mut builder = my::OptsBuilder::default();
    let port = config.port.clone().unwrap_or("3306".to_string());
    builder.user(config.user.clone()).pass(config.password.clone())
            .ip_or_hostname(config.host.clone())
            .tcp_port(port.parse().unwrap())
            .db_name(config.dbname.clone().unwrap().into());
    builder.into()
}

pub fn read_db(config: &DbConfig) -> Construction {
    let opts = get_opts(&config);
    let pool = my::Pool::new(opts).unwrap();
    let db_name = get_db_name_from_pool(&pool).unwrap();

    let table_names = collect_table_names(&pool);

    let mut tables = Vec::new();
    for tbl in table_names {
        let columns = collect_table_columns(&pool, &db_name, &tbl);
        let indexes = collect_indexes(&pool, &db_name, &tbl);
        let foreign_keys = collect_foregin_keys(&pool, &db_name, &tbl);
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

pub fn get_db_name_from_pool(pool: &my::Pool) -> Option<String> {
    match pool.prep_exec("SELECT database() AS db_name", ()).unwrap().next() {
        Some(Ok(row)) => my::from_row(row),
        _ => None
    }
}

/// テーブル一覧を取得する
pub fn collect_table_names(pool: &my::Pool) -> Vec<String> {
    pool.prep_exec("show tables", ()).map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let tbl_name = my::from_row(row);

            tbl_name
        }).collect()
    }).unwrap()
}

pub fn collect_table_columns(pool: &my::Pool, db_name: &String, table_name: &String) -> Vec<Column> {
    let query = r"
    SELECT column_name
        , column_type
        , column_key
        , extra
        , column_default
        , is_nullable
    FROM information_schema.columns c
    WHERE c.table_schema = ?
    AND c.table_name = ?
    ORDER BY ordinal_position";
    pool.prep_exec(query, (db_name, table_name)).map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (c_name, c_type, c_key, c_extra, c_default, c_is_nullable) : (String, String, String, String, Option<String>, String)
            = my::from_row(row);
            let is_primary = if c_key == "PRI".to_string() { true } else { false };
            Column {
                name: c_name,
                column_type: c_type,
                key: c_key,
                extra: c_extra,
                default: c_default,
                not_null: if c_is_nullable.to_lowercase() == "true" { false } else { true },
                is_primary: is_primary,
            }
        }).collect()
    }).unwrap()
}

pub fn collect_indexes(pool: &my::Pool, db_name: &String, table_name: &String) -> Vec<Index> {
    let query = r"
        SELECT index_name
            , column_name
            , seq_in_index
        FROM information_schema.statistics
        WHERE table_schema = ?
        AND table_name = ?";

    pool.prep_exec(query, (db_name, table_name)).map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (index_name, column_name, _seq_in_index) : (String, String, i32) = my::from_row(row);
            Index {
                name: index_name,
                column_name: column_name,
            }
        }).collect()
    }).unwrap()
}

pub fn collect_foregin_keys(pool: &my::Pool, db_name: &String, table_name: &String) -> Vec<ForeignKey> {
    let query = r"
        SELECT constraint_name
            , column_name
            , referenced_table_name
            , referenced_column_name
        FROM information_schema.key_column_usage
        WHERE constraint_schema = ?
        AND table_name = ?
        AND constraint_name <> 'PRIMARY'
        ORDER BY CONSTRAINT_NAME";
    pool.prep_exec(query, (db_name, table_name)).map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (constraint_name, column_name, referenced_table_name, referenced_column_name)
                 = my::from_row(row);
            ForeignKey {
                constraint_name: constraint_name,
                column_name: column_name,
                referenced_table_name: referenced_table_name,
                referenced_column_name: referenced_column_name,
            }
        }).collect()
    }).unwrap()
}
