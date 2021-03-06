use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::input::MouseTerminal;
use termion::screen::AlternateScreen;

use tui::Terminal;
use tui::style::{Color, Style, Modifier};
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Tabs, Widget, Paragraph, Text};

use crate::events::event::{Event, Events};

use std::fmt::Display;

struct TuiErr {
    tabs: Vec<String>,
    current: String,
    index: usize,
    data_map: HashMap<String, Vec<String>>,
}

impl TuiErr {
    fn new<E>(err: E) -> Self
    where
        E: Display,
    {
        let err = format!("{}", err);

        let mut data_map = HashMap::new();
        data_map.insert("Error".to_string(), vec![err.to_string()]);

        TuiErr {
            tabs: vec!["Error".to_string()],
            current: "Error".to_string(),
            index: 0,
            data_map,
        }
    }
}

fn listener(socket: &PathBuf, app_state: &Arc<Mutex<AppState>>) {
    let mut stream = UnixStream::connect(socket).unwrap();

    match stream.write_all(b"client\n") {
        Ok(val) => val,
        Err(err) => {
            let mut app_state = app_state.lock().unwrap();
            app_state.update_from_err(TuiErr::new(&err));
            return;
        }
    };

    for line in BufReader::new(stream).lines() {
        let mut app_state = app_state.lock().unwrap();
        if app_state.end {
            break;
        }

        let line = match line {
            Ok(val) => val,
            Err(err) => {
                app_state.update_from_err(TuiErr::new(&err));
                return;
            }
        };

        let split_vec = line.split(' ').collect::<Vec<&str>>();
        let (id, contents) = (split_vec[0], split_vec[2]);
        let mut contents = contents.to_string();
        contents.push('\n');

        if !app_state.tabs.contains(&id.to_string()) {
            app_state.tabs.push(id.to_owned());
        }

        let current_vec = app_state
            .data_map
            .entry(String::from(id))
            .or_insert_with(Vec::new);

        current_vec.push(contents);

        // initial receive go to first tab
        if app_state.current.is_empty() {
            app_state.current = id.to_owned();
        }
    }
}

struct AppState {
    index: usize,
    current: String,
    tabs: Vec<String>,
    data_map: HashMap<String, Vec<String>>,
    end: bool,
}

impl AppState {
    fn new() -> Self {
        AppState {
            index: 0,
            tabs: Vec::new(),
            current: String::new(),
            data_map: HashMap::new(),
            end: false,
        }
    }

    fn update_from_err(&mut self, err_obj: TuiErr) {
        self.index = err_obj.index;
        self.current = err_obj.current;
        self.tabs = err_obj.tabs;
        self.data_map = err_obj.data_map;
    }

    fn update_state(
        &mut self,
        next_tab: Option<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let next_tab = match next_tab {
            Some(val) => val,
            None => self.index,
        };

        if next_tab == self.index {
            return Ok(());
        } else if next_tab > self.tabs.len() {
            return Err(Box::from("no tab"));
        }

        let tab_str = match self.tabs.get(next_tab) {
            Some(val) => val,
            None => return Err(Box::from("no tab")),
        };

        self.current = tab_str.to_owned();
        self.index = next_tab;

        Ok(())
    }

    fn next(&mut self) {
        if self.tabs.len() == 1 {
            return;
        }

        let index = (self.index + 1) % self.tabs.len();

        if let Err(err) = self.update_state(Some(index)) {
            eprintln!("{}", err);
        };
    }

    fn previous(&mut self) {
        if self.tabs.len() == 1 {
            return;
        }

        let index = if self.index > 0 {
            self.index - 1
        } else {
            self.tabs.len() - 1
        };

        if let Err(err) = self.update_state(Some(index)) {
            eprintln!("{}", err);
        };
    }
}

pub struct TuiApp {
    socket_path: PathBuf,
    app: Arc<Mutex<AppState>>,
}

impl TuiApp {
    pub fn new(socket_path: PathBuf) -> Self {
        TuiApp {
            socket_path,
            app: Arc::new(Mutex::new(AppState::new())),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // for the thread
        let socket_path = self.socket_path.clone();
        let app_state = self.app.clone();

        thread::spawn(move || {
            listener(&socket_path, &app_state);
        });

        if let Err(err) = self.tui_start() {
            eprintln!("{}", err);
        }

        Ok(())
    }

    fn tui_start(&self) -> Result<(), Box<dyn Error>> {
        let events = Events::new();

        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.hide_cursor()?;

        let word_style = Style::default().fg(Color::Cyan);
        let word_style_hl = Style::default().fg(Color::Yellow);

        let block = Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().modifier(Modifier::Bold));

        loop {
            let size = terminal.size()?;

            terminal.resize(size)?;

            terminal.draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [Constraint::Length(3), Constraint::Min(0)].as_ref(),
                    )
                    .split(size);

                Block::default()
                    .style(Style::default().bg(Color::White))
                    .render(&mut f, size);

                let (tabs, index): (Vec<String>, usize) = self.get_tab_info();

                Tabs::default()
                    .block(Block::default().borders(Borders::ALL).title("Tabs"))
                    .titles(tabs.as_ref())
                    .select(index)
                    .style(word_style)
                    .highlight_style(word_style_hl)
                    .render(&mut f, chunks[0]);

                let text = self.get_text_widgets();

                Paragraph::new(text.iter())
                    .block(block.title("stdin"))
                    .alignment(Alignment::Left)
                    .render(&mut f, chunks[1]);
            })?;

            match events.next()? {
                Event::Input(input) => match input {
                    Key::Char('q') => {
                        let mut app_state = self.app.lock().unwrap();
                        app_state.end = true;
                        break;
                    }
                    Key::Right => self.next(),
                    Key::Left => self.previous(),
                    _ => {}
                },
                Event::Tick => {}
            };
        }
        Ok(())
    }

    fn get_tab_info(&self) -> (Vec<String>, usize) {
        let app_state = self.app.lock().unwrap();

        let (tabs, index) = if app_state.tabs.is_empty() {
            (vec!["None".to_string()], 0)
        } else {
            (app_state.tabs.to_owned(), app_state.index)
        };

        (tabs, index)
    }

    fn get_text_widgets(&self) -> Vec<Text> {
        let mut app_state = self.app.lock().unwrap();
        let current_key = app_state.current.to_owned();
        let current_vec = app_state.data_map.get_mut(&current_key);

        let mut none = vec!["None".to_string()];

        let text = if current_vec.is_some() {
            current_vec.unwrap()
        } else {
            &mut none
        };

        text.iter()
            .map(|val| Text::raw(val.to_string()))
            .collect::<Vec<Text>>()
    }

    fn next(&self) {
        let mut app_state = self.app.lock().unwrap();
        app_state.next();
    }

    fn previous(&self) {
        let mut app_state = self.app.lock().unwrap();
        app_state.previous();
    }
}
