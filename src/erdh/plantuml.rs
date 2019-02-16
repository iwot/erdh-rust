use super::erdh_data::Construction;
use super::erdh_data::Connection;
use std::io::{Write};

pub fn write_puml<W: Write>(cons: &Construction, writer: &mut W, groups: Option<Vec<String>>) -> Result<(), Box<std::error::Error>> {
    // let mut writer = BufWriter::new(File::create(path)?);

    writer.write("@startuml\n".as_bytes())?;

    let mut usable_tables = vec![];

    // グループ一覧
    let mut gorups = vec![];
    for tbl in &cons.tables {
        if let Some(group_vec) = &groups {
            if group_vec.len() > 0 {
                if !group_vec.contains(&tbl.group) {
                    continue;
                }
            }
        }
        usable_tables.push(&tbl.table);
        if !gorups.contains(&&tbl.group) {
            gorups.push(&tbl.group);
        }
    }

    for group in gorups {
        let iter = cons.tables.iter().filter_map(|t| if t.group == *group { Some(t) } else { None });
        let mut cnt = 0;
        for table in iter {
            if cnt == 0 {
                writer.write(format!("package \"{}\" as {} ", group, group).as_bytes())?;
                writer.write("{\n".as_bytes())?;
            }

            writer.write("  ".as_bytes())?;
            writer.write(format!("entity \"{}\" as {} <<D,TRANSACTION_MARK_COLOR>>", table.table, table.table).as_bytes())?;
            writer.write(" {\n".as_bytes())?;

            let mut column_cnt = 0;
            let max_count = 3;
            let mut absent_count = 0;
            for column in &table.columns {
                column_cnt += 1;
                if column_cnt > max_count {
                    absent_count += 1;
                    continue;
                }
                writer.write("    ".as_bytes())?;
                if column.is_primary {
                    writer.write("+ ".as_bytes())?;
                    writer.write(column.name.as_bytes())?;
                    writer.write(" [PK]".as_bytes())?;
                    writer.write("\n".as_bytes())?;
                    
                    writer.write("    ".as_bytes())?;
                    writer.write("--".as_bytes())?;
                    writer.write("\n".as_bytes())?;
                } else {
                    writer.write(column.name.as_bytes())?;
                    writer.write("\n".as_bytes())?;
                }
            }
            if absent_count > 0 {
                writer.write(format!("    .. {} more ..\n", absent_count).as_bytes())?;
            }
            writer.write("  }\n".as_bytes())?;

            cnt += 1;
        }
        if cnt > 0 {
            writer.write("}\n".as_bytes())?;
        }
    }

    // ここで出力されるカーディナリティも対象グループにより取捨選択する。
    for table in &cons.tables {
        if !usable_tables.contains(&&table.table) {
            continue;
        }
        for ex_relation in &table.ex_relations {
            if !usable_tables.contains(&&ex_relation.referenced_table_name) {
                continue;
            }
            writer.write(table.table.as_bytes())?;
            writer.write("  ".as_bytes())?;
            writer.write(get_this_cardinality(&ex_relation.this_conn).as_bytes())?;
            writer.write("--".as_bytes())?;
            writer.write(get_that_cardinality(&ex_relation.that_conn).as_str().as_bytes())?;
            writer.write("  ".as_bytes())?;
            writer.write(ex_relation.referenced_table_name.as_bytes())?;
            writer.write("\n".as_bytes())?;
        }
    }

    writer.write("@enduml\n".as_bytes())?;
    
    Ok(())
}

fn get_this_cardinality(conn: &Connection) -> String {
    let result = match conn {
        Connection::One => "--",
        Connection::OnlyOne => "||",
        Connection::ZeroOrOne => "|o",
        Connection::Many => "}-",
        Connection::OneMore => "}|",
        Connection::ZeroMany => "}o",
    };

    result.to_string()
}

fn get_that_cardinality(conn: &Connection) -> String {
    let result = match conn {
        Connection::One => "--",
        Connection::OnlyOne => "||",
        Connection::ZeroOrOne => "o|",
        Connection::Many => "-{",
        Connection::OneMore => "|{",
        Connection::ZeroMany => "o{",
    };

    result.to_string()
}
