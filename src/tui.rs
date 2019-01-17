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
use tui::layout::{Constraint, Direction, Layout};
// use tui::layout::{Alignment, Constraint, Direction, Layout};
// use tui::widgets::{Block, Borders, Tabs, Widget, Paragraph,  List, Text};
use tui::widgets::{Block, Borders, Tabs, Widget, List, Text};

use crate::events::event::{Event, Events};

// const STATIC_NONE: &str = "None";

fn listener(socket: &PathBuf, app_state: &Arc<Mutex<AppState>>) {
    let mut stream = UnixStream::connect(socket).unwrap();

    // tell the daemon we want to connect
    stream.write_all(b"client\n").unwrap();

    for line in BufReader::new(stream).lines() {
        let line = line.unwrap();

        let split_vec = line.split(' ').collect::<Vec<&str>>();
        let (id, contents) = (split_vec[0], split_vec[2]);
        let mut contents = contents.to_string();
        contents.push('\n');

        let mut app_state = app_state.lock().unwrap();

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
            app_state.current = id.to_string();
            app_state.index = 0;
        }
    }
}

#[allow(dead_code)]
struct AppState {
    index: usize,
    current: String,
    tabs: Vec<String>,
    data_map: HashMap<String, Vec<String>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            index: 0,
            tabs: Vec::new(),
            current: String::new(),
            data_map: HashMap::new(),
        }
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
        let index = (self.index + 1) % self.tabs.len();

        if let Err(err) = self.update_state(Some(index)) {
            eprintln!("{}", err);
        };
    }

    fn previous(&mut self) {
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

        /*
                let app_state = match self.app.lock() {
                    Ok(val) => val,
                    Err(err) => panic!("{}", err),
                };

                format!("tabs: {:?}", app_state.tabs);
                format!("data: {:?}", app_state.data_map);
                format!("current: {:?}", app_state.current);
                format!("index: {:?}", app_state.index);
                p
        */

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
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("The Tabs"),
                    )
                    .titles(tabs.as_ref())
                    .select(index)
                    .style(word_style)
                    .highlight_style(word_style_hl)
                    .render(&mut f, chunks[0]);

                let text: Vec<Text> = self.get_text_widgets();

                List::new(text.iter())
                    .block(block.title("words fool"))
                    .render(&mut f, chunks[1]);
            })?;

            match events.next()? {
                Event::Input(input) => match input {
                    Key::Char('q') => break,
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

    fn get_text_widgets(&self) -> Vec<&Text> {
        let mut app_state = self.app.lock().unwrap();
        let current_key = app_state.current.to_owned();
        let current_vec = app_state.data_map.get_mut(&current_key);

        let mut none = vec!["None".to_string()];

        let text = if current_vec.is_some() {
            &mut current_vec.unwrap()
        } else {
            &mut none
        };

        text.iter().map(&Text::raw).collect::<Vec<Text>>()
    }

    /*
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
    */

    fn next(&self) {
        let mut app_state = self.app.lock().unwrap();
        app_state.next();
    }

    fn previous(&self) {
        let mut app_state = self.app.lock().unwrap();
        app_state.previous();
    }
}
