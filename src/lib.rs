use rusqlite::{Connection,OpenFlags};
use std::{sync::Arc, time::Duration};
use chrono::{Datelike, NaiveTime, Utc};


//////////////////////////////////////////////////////
// ERRORS

#[derive(Debug)]
pub enum AppError {
    IcedError(Arc<iced::Error>),
    RSQLError(Arc<rusqlite::Error>),
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

// Actually not necessary
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
            AppError::IcedError(err) => write!(f, "Iced error: {}", err),
            AppError::RSQLError(err) => write!(f, "Rusqlite error: {}", err),
        }
    }
}


//////////////////////////////////////////////////////


// Functions to display today's agenda
pub fn display_agenda(conn: &Connection) -> (String,String) {
    let tomorrow = Utc::now() + Duration::from_secs(24*60*60);
    
    let date_today = format!("{:02}/{:02}",
                        Utc::now().day(),Utc::now().month());
    let tasks_today = db_reader(conn,&date_today).unwrap();
    
    let date_tomorrow = format!("{:02}/{:02}",
                        tomorrow.day(),tomorrow.month());
    let tasks_tomorrow = db_reader(conn,&date_tomorrow).unwrap();

    let agenda_today = vec![
        format!("{date_today}   Agenda for today: "),
        "_____________________________________________________________".to_string(),
        tasks_today
        ];

    let agenda_tomorrow = vec![
        format!("{date_tomorrow}   Agenda for tomorrow: "),
        "_____________________________________________________________".to_string(),
        tasks_tomorrow
        ];
    
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

    // Verify there is at least one entry
    if db_verify(conn, x)? == false {
        return Ok("NONE".to_string())
    }
    
    let mut stmt = conn.prepare("SELECT * FROM events WHERE date = ?1")?;
    let mut rows = stmt.query(&[&x])?;
    // In case there are MULTIPLE entries, concatenate the rows
    let mut activities: Vec<Vec<String>> = vec![]; 
    while let Ok(Some(row)) = rows.next() {
        
        let mut task: Vec<String> = vec![];
        let hour:String = row.get(2)?;
        if hour == "_" {
            task.push("Daylong".to_string());
        } else  {
            task.push(hour_padding(row.get(2)?));    
        };

        let a: i64 = row.get(0)?;
        task.push(format!("[{a}]"));
        task.push(row.get(3)?);        

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


pub fn db_writer(conn: &Connection, date: String, hour: String, task: String) -> Result<(), AppError> {
    // Insert rows into the table
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO events (date, hour, task) VALUES (?1, ?2, ?3)")?;
    stmt.execute((date, hour, task))?;

    Ok(())
}

pub fn db_verify_eraser(conn: &Connection, date: &String, id: &String) -> bool {

    // Check whether the entry does not exist
    let mut stmt = conn
        .prepare("SELECT * FROM events WHERE date = ?1 AND id = ?2").unwrap();
    let mut rows = stmt.query(&[&date, &id]).unwrap();
    if rows.next().unwrap().is_some() {true}
    else {false}
}

pub fn db_eraser(conn: &Connection, date: String, id: String) -> Result<(), AppError> {

    let mut stmt = conn.prepare(
        "DELETE FROM events WHERE date = ?1 AND id = ?2")?;

    stmt.execute((date, id))?; 
    Ok(())
}



pub fn db_setup(db_path: &str) -> Result<(), AppError> {
    let conn = Connection::open_with_flags(db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
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