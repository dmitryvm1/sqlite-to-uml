use std::error::Error;

use clap::Parser;
use drawio_uml::{builders::*, mxfile::Mxfile, fast_xml};
use crate::metadata::{list_tables, metadata_from_db};

mod metadata;

const DEFAULT_TABLE_WIDTH: i32 = 150;
const DEFAULT_OUTPUT_FILE_NAME: &str = "sql.drawio";
const DEFAULT_PAGE_NAME: &str = "Model";

#[derive(Parser)]
pub struct Cli {
    /// Path to the database
    #[arg(short, long)]
    database: String,
    /// Output file name, "sql.drawio" by default
    #[arg(short, long)]
    output: Option<String>,
    /// The width of each table
    #[arg(short, long)]
    width: Option<i32>,
    /// Page name, "Model" by default
    #[arg(short, long)]
    page: Option<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("Database: {}", cli.database);
    let connection = rusqlite::Connection::open(&cli.database)?;
    let tables = list_tables(&connection);
    println!("Tables: {:?}", tables);
    let mut diagram = new_diagram(&cli.page.unwrap_or(DEFAULT_PAGE_NAME.to_string()));
    let mut mxfile = Mxfile::default();
    let seq_id = SeqId::new();
    let mut x_offset = 10;
    let width = cli.width.unwrap_or(DEFAULT_TABLE_WIDTH);
    for table in tables {
        let table_metadata = metadata_from_db(&table, &connection);
        let mut cls = UMLClass::new(&table, "1", seq_id.clone(), width);
        for column in table_metadata.columns {
            cls.add_field(&format!("{}: {} {}", column.name, column.r#type, if column.notnull {"NOT NULL"}else {""}));
        }
        cls.set_position(x_offset, 30);
        diagram.mx_graph_model.root.elements.append(&mut cls.into_cells());
        x_offset += width + 20;
    }
    mxfile.diagrams.push(diagram);
    std::fs::write(cli.output.unwrap_or(DEFAULT_OUTPUT_FILE_NAME.to_string()), fast_xml::se::to_string(&mxfile).unwrap()).unwrap();
    Ok(())
}
