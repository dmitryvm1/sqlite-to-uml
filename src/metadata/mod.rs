use std::fmt::Display;

#[derive(Debug)]
pub struct TableMetadata {
    pub name: String,
    pub columns: Vec<ColumnMetadata>
}

#[derive(Debug)]
pub struct ColumnMetadata {
    pub name: String,
    pub r#type: LiteType,
    pub notnull: bool
}

#[derive(Debug)]
pub enum LiteType {
    Text,
    Blob,
    Integer,
    Real,
    Double,
    Varchar(Option<usize>),
    Boolean,
    Timestamp,
    BigInt,
    Date
}

impl From<&str> for LiteType {
    fn from(value: &str) -> Self {
        match value {
            "TEXT" => LiteType::Text,
            "BLOB" => LiteType::Blob,
            "INTEGER" => LiteType::Integer,
            "REAL" => LiteType::Real,
            "BOOLEAN" => LiteType::Boolean,
            "TIMESTAMP" => LiteType::Timestamp,
            "DATE" => LiteType::Date,
            "BIGINT" => LiteType::BigInt,
            "DOUBLE" => LiteType::Double,
            "VARCHAR" => LiteType::Varchar(None),
            _ => {
                if value.starts_with("VARCHAR(") {
                    let closing = value.find(")").unwrap();
                    return LiteType::Varchar(Some(value[8..closing].parse::<usize>().unwrap()))
                } else {
                    panic!("{}", value)
                }
            }
        }
    }
}

impl Display for LiteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteType::Text => f.write_str("TEXT"),
            LiteType::Blob => f.write_str("BLOB"),
            LiteType::Integer => f.write_str("INTEGER"),
            LiteType::Real => f.write_str("REAL"),
            LiteType::Double => f.write_str("DOUBLE"),
            LiteType::Varchar(n) => {
                match n {
                    Some(sz) => {
                        f.write_str("VARCHAR(")?;
                        f.write_fmt(format_args!("{}", sz))?;
                        f.write_str(")")
                    }
                    None => {
                        f.write_str("VARCHAR")
                    }
                }
            },
            LiteType::Boolean => f.write_str("BOOLEAN"),
            LiteType::Timestamp => f.write_str("TIMESTAMP"),
            LiteType::BigInt => f.write_str("BIGINT"),
            LiteType::Date => f.write_str("DATE"),
        }
    }
}

pub fn list_tables(connection: &rusqlite::Connection) -> Vec<String> {
    let mut stmt = connection.prepare(r#"SELECT name FROM sqlite_schema
            WHERE type ='table' AND 
            name NOT LIKE 'sqlite_%';"#).unwrap();
    let mut rows = stmt.query(()).unwrap();
    let mut list = Vec::new();
    while let Ok(Some(r)) = rows.next() {
        list.push(r.get(0).unwrap());
    }
    list
}

pub fn metadata_from_db(table_name: &str, connection: &rusqlite::Connection) -> TableMetadata {
    let mut stmt = connection.prepare(&format!(r#"SELECT cid, name, type, "notnull", dflt_value, pk FROM pragma_table_info("{}");"#, table_name)).unwrap();
    let mut rows = stmt.query(()).unwrap();
    let mut metadata = TableMetadata {
        columns: Vec::new(),
        name: table_name.to_string()
    };
    while let Ok(Some(row)) = rows.next() {
        metadata.columns.push(ColumnMetadata {
            name: row.get(1).unwrap(),
            r#type: LiteType::from(row.get::<_, String>(2).unwrap().as_str()),
            notnull: row.get::<_, bool>(3).unwrap()
        })
    }

    metadata
}