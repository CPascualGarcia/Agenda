use rusqlite::{Connection,OpenFlags};
use std::{sync::Arc, time::Duration};
use chrono::{Datelike, NaiveTime, Utc};


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


// Functions to display today's agenda
pub fn display_agenda() -> (String,String) {
    let tomorrow = Utc::now() + Duration::from_secs(24*60*60);
    
    // let (month_today,day_today) = (Utc::now().month(),Utc::now().day());
    // let (month_tomorrow,day_tomorrow) = (tomorrow.month(),tomorrow.day());

    let agenda_today = vec![
        format!("{0:?}/{1:?}   Agenda for today: ",Utc::now().day(),Utc::now().month()),
        "__________________________________________________".to_string(),
        " 9:00 - nothing to do".to_string()
        ];

    let agenda_tomorrow = vec![
        format!("{0:?}/{1:?}   Agenda for tomorrow: ",tomorrow.day(),tomorrow.month()),
        "__________________________________________________".to_string(),
        " 9:00 - nothing to do".to_string()
        ];
    
    // let agenda = vec!["Line_1".to_string(), "Line_2".to_string(), "Line_3".to_string()];
    (agenda_today.join("\n"),agenda_tomorrow.join("\n"))
}


//////////////////////////////////////////////////////
// Functions to manipulate the database

pub fn parser_input(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()    
}


pub fn db_reader(conn: &Connection, x: &str) -> Result<String, AppError> {

    // Verify the entry exists
    if db_verify(conn, x)? == false {
        // println!("Entry does not exist in database.");
        return Ok("NONE".to_string())
    }
    
    let mut stmt = conn.prepare("SELECT * FROM events WHERE date = ?1")?;
    let mut rows = stmt.query(&[&x])?;
    // let row = rows.mapped().iter();
    // In case there are MULTIPLE entries, concatenate the rows
    let mut activities: Vec<Vec<String>> = vec![]; 
    while let Ok(Some(row)) = rows.next() {
        
        let mut task: Vec<String> = vec![];
        let hour:String = row.get(1)?;
        if hour == "_" {
            task.push("Daylong".to_string());
        } else  {
            task.push(hour_padding(row.get(1)?));    
        };

        // activities.push(row.get(1)?);
        task.push(row.get(2)?);        
        // activities.push(task.join(" - "));
        activities.push(task);
    };

    activities.sort_by(|a, b| {
        match (a[0].parse::<NaiveTime>(), b[0].parse::<NaiveTime>()) {
            (Ok(a), Ok(b)) => a.cmp(&b),
            (Ok(_), Err(_)) => std::cmp::Ordering::Less,
            (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
            (Err(_), Err(_)) => a[0].cmp(&b[0]),
        }
    });

    let joined_schedule: Vec<String> = activities.iter()
            .map(|row| row.join(" - "))
            .collect();

    // Ok(activities.join("\n"))
    Ok(joined_schedule.join("\n"))
}


pub fn db_verify(conn: &Connection, x: &str) -> Result<bool, AppError> {

    // Check whether the entry does not exist
    let mut stmt = conn.prepare("SELECT * FROM events WHERE date = ?1")?;
    let mut rows = stmt.query(&[&x])?;
    if rows.next().unwrap().is_some() {Ok(true)}
    else {Ok(false)}
}

// OLD - Use as reference
// pub fn db_writer(conn: &Connection, buffer: String, x: usize) -> Result<(), AppError> {
//     // Insert rows into the table
//     let mut stmt = conn.prepare(
//         "INSERT OR REPLACE INTO tasks (id, task) VALUES (?1, ?2)")?;
//     stmt.execute((x, buffer))?;

//     Ok(())
// }

pub fn db_writer(conn: &Connection, date: String, hour: String, task: String) -> Result<(), AppError> {
    // Insert rows into the table
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO events (date, hour, task) VALUES (?1, ?2, ?3)")?;
    stmt.execute((date, hour, task))?;

    Ok(())
}

pub fn db_verify_eraser(conn: &Connection, date: &String, task: &String) -> bool {

    // Check whether the entry does not exist
    let mut stmt = conn
        .prepare("SELECT * FROM events WHERE date = ?1 AND task = ?2").unwrap();
    let mut rows = stmt.query(&[&date, &task]).unwrap();
    if rows.next().unwrap().is_some() {true}
    else {false}
}

pub fn db_eraser(conn: &Connection, date: String, task: String) -> Result<(), AppError> {

    let mut stmt = conn.prepare(
        "DELETE FROM events WHERE date = ?1 AND task = ?2")?;

    stmt.execute((date, task))?; 
    Ok(())
    // match stmt.execute((date, task)) {
    //     Ok(_) => {
    //         Ok(())
    //     },
    //     Err(e) => { 
    //         Err(AppError::RSQLError(Arc::new(e)))
    //     }
    // }
}



pub fn db_setup(db_path: &str) -> Result<(), AppError> {
    let conn = Connection::open_with_flags(db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;

    // Create basic table
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS 
    //     tasks (id INTEGER PRIMARY KEY, task TEXT NOT NULL)", 
    //     ())?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            date TEXT,
            hour TEXT,
            task TEXT
        )",
        (),
    )?;

    Ok(())
}

///////////////////////////////////////////////////
// Other functions

pub fn hour_padding(time: String) -> String {
    if time.len() < 5 {
        // format!("0{}",time)
        "0".to_string() + &time
    } else {
        time.to_string()
    }
}