use rusqlite::{Connection,OpenFlags};
use std::sync::Arc;


//////////////////////////////////////////////////////
// ERRORS

#[derive(Debug)]
pub enum AppError {
    // StdError(std::error::Error),
    IcedError(Arc<iced::Error>),
    RSQLError(Arc<rusqlite::Error>),
    // IO(io::ErrorKind)
}

impl From<iced::Error> for AppError {
    fn from(e: iced::Error) -> Self {
        AppError::IcedError(Arc::new(e))
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::RSQLError(Arc::new(e))
    }
}

// I still need to understand why I wrote this initially...
// impl Clone for AppError {
//     fn clone(&self) -> Self {
//         match self {
//             AppError::IcedError(err) => AppError::IcedError(err.clone()),
//             AppError::RSQLError(err) => AppError::RSQLError(err.clone()),
//             // AppError::IO(err) => AppError::IO(err.clone()),
//         }
//     }
// }

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // AppError::StdError(err) => write!(f, "Standard error: {}", err),
            AppError::IcedError(err) => write!(f, "Iced error: {}", err),
            AppError::RSQLError(err) => write!(f, "Rusqlite error: {}", err),
            // AppError::IO(err) => write!(f, "IO error: {}", err),
        }
    }
}


//////////////////////////////////////////////////////




//////////////////////////////////////////////////////
// Functions to manipulate the database

pub fn parser_input(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()    
}


pub fn db_reader(conn: &Connection, x: &usize) -> Result<String, AppError> {

    // Verify the entry exists
    if db_verify(conn, *x)? == false {
        // println!("Entry does not exist in database.");
        return Ok("NONE".to_string());
    }
    
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE id = ?1")?;
    let mut rows = stmt.query(&[&x])?;

    let Some(row) = rows.next()? else { return Ok("NONE".to_string()) };
    let task: String = row.get(1)?;

    Ok(task)
}


pub fn db_verify(conn: &Connection, x: usize) -> Result<bool, AppError> {

    // Check that the entry does not exist
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE id = ?1")?;
    let mut rows = stmt.query(&[&x])?;
    if rows.next().unwrap().is_some() {Ok(true)}
    else {Ok(false)}
}

pub fn db_writer(conn: &Connection, buffer: String, x: usize) -> Result<(), AppError> {
    // Insert rows into the table
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO tasks (id, task) VALUES (?1, ?2)")?;
    stmt.execute((x, buffer))?;

    Ok(())
}

pub fn db_setup(db_path: &str) -> Result<(), AppError> {
    let conn = Connection::open_with_flags(db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;

    // Create basic table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS 
        tasks (id INTEGER PRIMARY KEY, task TEXT NOT NULL)", 
        ())?;

    Ok(())
}