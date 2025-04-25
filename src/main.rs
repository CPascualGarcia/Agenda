
use iced::{Element,Length,Renderer,Task,Theme};
use iced::widget::{Button,Container,Space,Text,column,row,text_editor};


use rusqlite::{Connection,OpenFlags};

// extern crate Agenda;
use Agenda::*;
// mod my_lib;
// use my_lib::*;


// TO DO
// Formatting - adjust spces for the text editor
// Asynchronous functionalities



fn main() -> Result<(),AppError> {
    let db_path: &str = "TodoList.db"; // Prepare the path to the database
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
    query: String,
    result: String,
    result_add: String
}

#[derive(Debug,Clone)]
enum Message {
    TextEditorAction(text_editor::Action),
    TextEditorActionAdd(text_editor::Action),
    QueryDo,
    QueryChange
}



impl DBEditor {
    fn new(connection:Connection) -> (Self, Task<Message>) {
        (
            Self {
            db_conn: connection,
            content: text_editor::Content::with_text("Write here the no. of the line (e.g. 5)"),
            content_add: text_editor::Content::with_text("Write here as: <line no.> <task>"),
            
            query: String::new(),
            result: String::new(),
            result_add: String::new(),
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
            Message::QueryDo => {
                
                match self.query.trim().parse::<usize>() {
                    Ok(query_input) => {
                        self.result = db_reader(&self.db_conn, &query_input).unwrap();
                    },
                    Err(_) => {
                        self.result = "Error parsing query".to_string();
                    }
                };
                // Task::perform(async move{
                //     db_writer(&self.db_conn, "mustard", &self.content.as_text().parse::<usize>()).unwrap()},
                //      Message::TextEditorAction);
                
            },
            Message::QueryChange => {
                if parser_input(&self.query).len() < 2 {
                    self.result_add = "Invalid query".to_string();
                    ()
                } else {
                    let (line1, contents_line1) = &self.query.trim().split_once(' ').unwrap();
                    match (line1.parse::<usize>(), contents_line1.parse::<String>()) {                    
                        (Ok(line), Ok(contents_line)) => {
                            db_writer(&self.db_conn, contents_line, line).unwrap();
                            self.result_add = "New task added".to_string();
                        },
                        _ => {
                            self.result_add = "Unable to parse query".to_string();
                        }
                    }
                }
            }
        }

        iced::Task::none()
    }

    fn view(&self) -> Element<'_,Message> {
        // let query_input = 5 as usize;
        // let result = db_reader(&self.db_conn, &query_input).unwrap();
        // let result_holder = Text::new(result),

        // Verification of an entry
        let display = Text::new("Check task at given line number: ");
        
        let input = iced::widget::TextEditor::new(&self.content)
            .on_action(Message::TextEditorAction);

        let exec_button = Button::new("Search")
        .on_press(Message::QueryDo);

        let output = Text::new(&self.result);
        //
        
        // Addition/modification of an entry
        let display_add: Text<'_, Theme, Renderer> = Text::new("Add/overwrite task: ");

        let input_add: iced::widget::TextEditor<'_, _, Message> = iced::widget::TextEditor::new(&self.content_add)
            .on_action(Message::TextEditorActionAdd);

        let exec_button_add: iced::widget::Button<'_, Message, Theme, Renderer> = Button::new("Add")
        .on_press(Message::QueryChange);

        let output_add: Text<'_, Theme, Renderer> = Text::new(&self.result_add);
        //

        
        let layout = row![
            Space::with_width(Length::Fixed(4.0)),
            column![
                row![display, Space::with_width(Length::Fill), exec_button],
                input,
                output
                ],
            Space::with_width(Length::Fixed(8.0)),
            column![
                row![display_add,Space::with_width(Length::Fill),exec_button_add],
                input_add,
                output_add
                ],
            Space::with_width(Length::Fixed(4.0)),
            ];

        let header = Text::new("Welcome to the to-do list editor");
        let layout2 = column![header.center(), layout].align_x(iced::Alignment::Center);
        
        Container::new(layout2)
            // .align_x(iced::Center)
            .align_y(iced::Alignment::Center)
            // .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }
}








