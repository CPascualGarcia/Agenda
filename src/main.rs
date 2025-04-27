
// use std::clone;

use iced::{Element,Length,Renderer,Task,Theme};
use iced::widget::{Button,Container,Space,Text,column,row,text_editor};
use chrono::{NaiveDate,NaiveTime};

use rusqlite::{Connection,OpenFlags};

// extern crate Agenda;
use Agenda::*;


// TO DO
// Add year to the entries in db_writer and db_reader
// Add a delete button
// In db_eraser, verify that the code returns something if the asked entry does not exist
// Perhaps adapt content_add into a multi-box setup
// Asynchronous functionalities



fn main() -> Result<(),AppError> {
    let db_path: &str = "Agenda_CPG.db"; // Prepare the path to the database
    db_setup(db_path).unwrap();       // Set database
    let conn = Connection::open_with_flags(db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;
    

    // Set the app
    iced::application("Agenda", DBEditor::update, DBEditor::view)
    .theme(DBEditor::theme)
    .run_with(|| DBEditor::new(conn))?;
    Ok(())
}



struct DBEditor {
    db_conn: Connection,
    content: text_editor::Content,
    content_add: text_editor::Content,
    // content_erase: text_editor::Content,
    query: String,
    result_check: String,
    result_add: String,
    // result_erase: String,

    // Agenda 
    agenda_today:    String,
    agenda_tomorrow: String
}

#[derive(Debug,Clone)]
enum Message {
    TextEditorAction(text_editor::Action),
    TextEditorActionAdd(text_editor::Action),
    // TextEditorActionErase(text_editor::Action),
    QueryDo,
    QueryChange
}



impl DBEditor {
    fn new(connection:Connection) -> (Self, Task<Message>) {
        (
            Self {
            db_conn: connection,
            content: text_editor::Content::with_text("Input as: <DD/MM>"),
            content_add: text_editor::Content::with_text("Input as: <DD/MM> <HH:mm (optional)> <task>"),
            // content_erase:: text_editor::Content::with_text("Input as: <DD/MM> <task>"),

            query:        String::new(),
            result_check: String::new(),
            result_add:   String::new(),
            // result_erase: String::new(),

            agenda_today: display_agenda().0,
            agenda_tomorrow: display_agenda().1
        },
        // Task::perform(future, Message::TextAdded)
        Task::none()
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TextEditorAction(action) => {
                self.content.perform(action);
                self.query = self.content.text();
            },
            Message::TextEditorActionAdd(action) => {
                self.content_add.perform(action);
                self.query = self.content_add.text();
            },
            // Message::TextEditorActionErase(action) => {
            //     self.content_erase.perform(action);
            //     self.query = self.content_erase.text();
            // },
            Message::QueryDo => {
                let input_query = self.query.trim().to_owned()+"/2025";
                match NaiveDate::parse_from_str(&input_query, "%d/%m/%Y") {
                    Ok(_) => {
                        let date = self.query.trim().to_string();
                        self.result_check = db_reader(&self.db_conn, &date).unwrap();
                    },
                    Err(_) => {
                        self.result_check = "Error parsing query".to_string();
                    }
                }              
            },
            Message::QueryChange => {
                let contents =  parser_input(&self.query);
                if  contents.len()< 2 {
                    self.result_add = "Invalid query".to_string()
                };
                // Match case to parse the hour
                match NaiveTime::parse_from_str(&contents[1], "%H:%M") {
                    Ok(_) => {
                        let (date, hour, task) = 
                            (contents[0].clone(), contents[1].clone(), contents[2..].join(" ").clone());
                        db_writer(&self.db_conn, date, hour, task).unwrap();
                        self.result_add = "New timed task added".to_string();
                    }
                    Err(_) => {
                        let (date, task) = (contents[0].clone(), contents[1..].join(" ").clone());
                        db_writer(&self.db_conn, date, "_".to_string(),task).unwrap();
                        self.result_add = "New untimed task added".to_string();
                    }
                }
            }                
                    
            //////////////////////////////// COPY THIS EXAMPLE
            // fn from_str(s: &str) -> Result<Self, Self::Err> {
            // let mut split = s.split(',')
            // let (Some(name), Some(age), None) = (split.next(), split.next(), split.next()) else {
            //     //                      ^^^^ there should be no third element
            //     return Err(ParsePersonError::BadLen);
            // };
            ///////////////////////////////

        //     let // (line1, contents_line1) = &self.query.trim().split_once(' ').unwrap();
        //     match (line1.parse::<String>(), contents_line1.parse::<String>()) {                    
        //         (Ok(line), Ok(contents_line)) => {
        //             db_writer(&self.db_conn, contents_line, line,"".to_string() ).unwrap();
        //             self.result_add = "New task added".to_string();
        //         },
        //         _ => {
        //             self.result_add = "Unable to parse query".to_string();
        //         }
        //     }
        // }
        };

        iced::Task::none()
    }

    fn view(&self) -> Element<'_,Message> {
        // let query_input = 5 as usize;
        // let result_check = db_reader(&self.db_conn, &query_input).unwrap();
        // let result_holder = Text::new(result_check),

        // Verification of an entry
        let display = Text::new("Check task at given day: ");
        
        let input_check = iced::widget::TextEditor::new(&self.content)
            .on_action(Message::TextEditorAction);

        let exec_button = Button::new("Search")
        .on_press(Message::QueryDo);

        let output_check = Text::new(&self.result_check);
        //
        
        // Addition/modification of an entry
        let display_add: Text<'_, Theme, Renderer> = Text::new("Add/overwrite task: ");

        let input_add: iced::widget::TextEditor<'_, _, Message> = iced::widget::TextEditor::new(&self.content_add)
            .on_action(Message::TextEditorActionAdd);

        let exec_button_add: iced::widget::Button<'_, Message, Theme, Renderer> = Button::new("Add")
        .on_press(Message::QueryChange);

        let output_add: Text<'_, Theme, Renderer> = Text::new(&self.result_add);
        //

        // Erasure of a task
        let display_erase: Text<'_, Theme, Renderer> = Text::new("Erase task: ");

        // let input_erase: iced::widget::TextEditor<'_, _, Message> = iced::widget::TextEditor::new(&self.content_erase)
        //     .on_action(Message::TextEditorActionErase);

        // let exec_button_erase: iced::widget::Button<'_, Message, Theme, Renderer> = Button::new("Erase").on_press(Message::QueryErase);

        // let output_erase: Text<'_, Theme, Renderer> = Text::new(&self.result_erase);

        ///////////////////////////////////////////////////////////////////////////////////////////
        /////////////////////
        // Agenda for today

        // let display_today: Text<'_, Theme, Renderer>  = Text::new("Agenda for today: ");
        let output_today: Text<'_, Theme, Renderer>  = Text::new(&self.agenda_today).height(200.);

        let today: iced::widget::Column<'_, Message> = column![
            // display_today,
            output_today
        ];

        /////////////////////
        // Agenda for tomorrow

        // let display_tomorrow: Text<'_, Theme, Renderer>  = Text::new("Agenda for tomorrow: "); 
        let output_tomorrow: Text<'_, Theme, Renderer>  = Text::new(&self.agenda_tomorrow).height(200.);

        let tomorrow: iced::widget::Column<'_, Message> = column![
            // display_today,
            output_tomorrow
        ];
        
        ///////////////////////////////////////////////////////////////////////////////////////////
        
        let layout = row![
            Space::with_width(Length::Fixed(6.0)),
            column![
                today,
                Space::with_height(Length::Fixed(10.0)),
                row![display, Space::with_width(Length::Fill), exec_button],
                input_check,
                output_check,
                display_erase
                ],
            Space::with_width(Length::Fixed(10.0)),
            column![
                tomorrow,
                Space::with_height(Length::Fixed(10.0)),
                row![display_add,Space::with_width(Length::Fill),exec_button_add],
                input_add,
                output_add
                ],
            Space::with_width(Length::Fixed(6.0)),
            ];

        let header = Text::new("Welcome to the agenda!").size(40);
        let layout2 = column![header.center(), Space::with_height(Length::Fixed(15.0)), layout].align_x(iced::Alignment::Center);
        
        Container::new(layout2)
            // .align_x(iced::Center)
            .align_y(iced::Alignment::Center)
            // .width(Length::Fill)
            // .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }
}








