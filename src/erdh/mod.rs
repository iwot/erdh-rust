use super::config::{Config, SourceType};
use super::config::db_config::DbConfig;
use super::config::extra_config::ExtraConfig;

pub mod erdh_data;
pub mod plantuml;

pub fn get_construction(config: &Config) -> Option<erdh_data::Construction> {
    match config.source {
        SourceType::YAML => Some(
            erdh_data::Construction::from_yaml_file(&config.source_from).unwrap()
            ),
        SourceType::MySQL => {
            let db_config = DbConfig::from_yaml_file(&config.source_from);
            if db_config.is_ok() {
                Some(super::db::mysql::read_db(&db_config.unwrap()))
            } else {
                None
            }
        },
        SourceType::PostgreSQL => {
            let db_config = DbConfig::from_yaml_file(&config.source_from);
            if db_config.is_ok() {
                Some(super::db::postgres::read_db(&db_config.unwrap()))
            } else {
                None
            }
        },
        SourceType::SQLite => {
            let db_config = DbConfig::from_yaml_file(&config.source_from);
            if db_config.is_ok() {
                Some(super::db::sqlite::read_db(&db_config.unwrap()))
            } else {
                None
            }
        },
    }
}

pub fn apply_ex_info(cons: &mut erdh_data::Construction, ex: &ExtraConfig) {
    let max = cons.tables.len();
    for i in 0..max {
        for et in &ex.tables {
            if cons.tables[i].table == et.table {
                if let Some(is_master) = &et.is_master {
                    cons.tables[i].is_master = Some(is_master.clone());
                }
                if let Some(group) = &et.group {
                    cons.tables[i].group = group.clone();
                }
                if let Some(relations) = &et.relations {
                    let max_r = cons.tables[i].ex_relations.len();
                    for r in relations {
                        let mut found = false;
                        for j in 0..max_r {
                            if cons.tables[i].ex_relations[j].referenced_table_name == r.referenced_table_name {
                                cons.tables[i].ex_relations[j] = r.get_clone();
                                found = true;
                            }
                        }

                        if !found {
                            cons.tables[i].ex_relations.push(r.get_clone());
                        }
                    }
                }
            }
        }
    }
}
