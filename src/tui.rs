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

// const STATIC_NONE: &str = "None";

fn listener(socket: &PathBuf, app_state: &Arc<Mutex<AppState>>) {
    let mut stream = UnixStream::connect(socket).unwrap();

    stream.write_all(b"client\n").unwrap();

    for line in BufReader::new(stream).lines() {
        let line = line.unwrap();

        let split_vec = line.split(' ').collect::<Vec<&str>>();
        let (id, contents) = (split_vec[0], split_vec[2]);

        let mut app_state = app_state.lock().unwrap();

        let current_vec = app_state
            .data_map
            .entry(String::from(id))
            .or_insert_with(Vec::new);

        current_vec.push(contents.to_string());

        println!("{:?}", current_vec);
    }
}

#[allow(dead_code)]
struct AppState {
    tabs: Vec<String>,
    index: usize,
    data_map: HashMap<String, Vec<String>>,
    current: String,
}

impl AppState {
    fn new() -> Self {
        // tabs: vec![STATIC_NONE.to_owned()],
        AppState {
            current: String::new(),
            data_map: HashMap::new(),
            tabs: Vec::new(),
            index: 0,
        }
    }

    // fn update_state() {}

    // fn next(&mut self) -> Result<(), Box<dyn Error>> { Ok(()) }
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

        let mut dumy = &mut vec![];
        let mut err_vec: Vec<String> = vec![];

        loop {
            let size = terminal.size()?;

            terminal.resize(size)?;

            let mut app_state = self.app.lock().unwrap();
            // let mut app_state = self.app.lock().unwrap();

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

                Tabs::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("The Tabs"),
                    )
                    .titles(app_state.tabs.as_ref())
                    .select(app_state.index)
                    .style(word_style)
                    .highlight_style(word_style_hl)
                    .render(&mut f, chunks[0]);

                /*
                let text = vec![
                    Text::raw(format!("tabs: {:?}", app_state.tabs)),
                    Text::raw(format!("data: {:?}", app_state.data_map)),
                    Text::raw(format!("current: {:?}", app_state.current)),
                    Text::raw(format!("index: {:?}", app_state.index)),
                ];

                Paragraph::new(text.iter())
                    .block(block.title("what"))
                    .alignment(Alignment::Left)
                    .render(&mut f, chunks[1]);

                */
                let current_key = app_state.current.to_owned();

                let current_vec: &mut Vec<String> =
                    match app_state.data_map.get_mut(&current_key) {
                        Some(val) => val,
                        None => {
                            err_vec.push("Nothing in current_vec".to_string());
                            &mut dumy
                        }
                    };

                // defitly a better way
                let text: Vec<Text> =
                    if !current_key.is_empty() && !current_vec.is_empty() {
                        if !err_vec.is_empty() {
                            current_vec.append(&mut err_vec);
                        }
                        current_vec.iter().map(Text::raw).collect::<Vec<Text>>()
                    } else if !err_vec.is_empty() {
                        err_vec.iter().map(Text::raw).collect::<Vec<Text>>()
                    } else {
                        vec![Text::raw("none here")]
                    };

                Paragraph::new(text.iter())
                    .block(block.title("words fool"))
                    .alignment(Alignment::Left)
                    .render(&mut f, chunks[1]);
            })?;

            err_vec.clear();

            if let Event::Input(input) = events.next()? {
                if let Key::Char('q') = input {
                    break;
                }
            }
        }
        Ok(())
    }
}
