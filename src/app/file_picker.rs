use clap::Parser;
use iced::{
    widget::{button, row, text, text_input, Column, Scrollable},
    Color,
};
use regex::Regex;

use std::{fs, path::PathBuf};

use crate::app::{Message, PADDING};

use super::accounts::Accounts;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// load FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    load: Option<String>,

    /// Create a new FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    new: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FilePicker {
    current: PathBuf,
    filename: String,
    error: String,
    show_hidden_files: bool,
}

impl FilePicker {
    pub fn load_or_new_file() -> Option<(Accounts, PathBuf)> {
        let args = Args::parse();

        if let Some(arg) = args.load {
            let path_buf = PathBuf::from(arg);
            let mut accounts = Accounts::load(&path_buf)
                .unwrap_or_else(|err| panic!("error loading {:?}: {}", &path_buf, err));
            accounts.check_monthly();
            accounts.save(&path_buf);
            return Some((accounts, path_buf));
        }
        if let Some(arg) = args.new {
            let path_buf = PathBuf::from(arg);
            let accounts = Accounts::new();
            accounts
                .save_first(&path_buf)
                .unwrap_or_else(|err| panic!("error creating {:?}: {}", &path_buf, err));
            return Some((accounts, path_buf));
        }
        None
    }

    pub fn new() -> Self {
        let path = match fs::canonicalize(".") {
            Ok(path) => path,
            Err(_) => PathBuf::from("."),
        };

        FilePicker {
            current: path,
            filename: String::new(),
            error: String::new(),
            show_hidden_files: false,
        }
    }

    pub fn new_file(&mut self, mut file: PathBuf) -> Option<(Accounts, PathBuf)> {
        if file.as_os_str().is_empty() {
            return None;
        }

        let mut file_path = self.current.clone();
        file.set_extension("json");
        file_path.push(file);
        let accounts = Accounts::new();
        if let Err(err) = accounts.save_first(&file_path) {
            self.error = format!("error creating {:?}: {}", &file_path, err);
            return None;
        }
        Some((accounts, file_path))
    }

    pub fn load_file(&mut self, file_path: &PathBuf) -> Option<Accounts> {
        match Accounts::load(file_path) {
            Ok(mut accounts) => {
                accounts.check_monthly();
                accounts.save(file_path);
                Some(accounts)
            }
            Err(err) => {
                self.error = format!("error loading {:?}: {}", &file_path, err);
                None
            }
        }
    }

    pub fn change_dir(&mut self, path_buf: PathBuf) {
        self.current = path_buf;
        self.error = String::new();
    }

    pub fn change_file_name(&mut self, file: &str) {
        self.filename = file.trim().to_string();
        self.error = String::new();
    }

    pub fn view(&self) -> Column<Message> {
        let mut col = Column::new();
        if !self.error.is_empty() {
            col = col.push(row![text(&self.error)].padding(PADDING))
        }
        if let Some(dir) = self.current.parent() {
            col = col.push(
                row![button(text(dir.display())).on_press(Message::ChangeDir(dir.into()))]
                    .padding(PADDING),
            );
        }
        col = col.push(row![text(self.current.to_str().unwrap())].padding(PADDING));
        col = col.push(Scrollable::new(self.files().unwrap()));
        col = col.push(
            row![
                text_input("filename", &self.filename)
                    .on_input(Message::ChangeFileName)
                    .on_submit(Message::NewFile(PathBuf::from(&self.filename))),
                text(".json")
            ]
            .padding(PADDING),
        );
        col
    }

    fn files(&self) -> Result<Column<Message>, std::io::Error> {
        let is_json = Regex::new(r".json$").unwrap();
        let mut col = Column::new();
        let mut dirs = Vec::new();
        for entry in fs::read_dir(&self.current)? {
            let dir = entry?;
            dirs.push(dir);
        }
        dirs.sort_by_key(|dir| dir.file_name());

        for dir in dirs {
            let file_path = dir.path();
            let file_type = dir.file_type()?;
            let file_name = dir.file_name();
            let file_name = match file_name.into_string() {
                Ok(s) => {
                    if !self.show_hidden_files && s.starts_with('.') {
                        continue;
                    }
                    s
                }
                Err(_) => continue,
            };

            if file_type.is_file() && is_json.is_match(&file_name) {
                col = col.push(
                    row![button(text(&file_name))
                        .style(iced::theme::Button::Custom(Box::new(GreenButton)))
                        .on_press(Message::LoadFile(file_path))]
                    .padding(PADDING),
                );
            } else if file_type.is_dir() {
                col = col.push(
                    row![button(text(&file_name)).on_press(Message::ChangeDir(file_path))]
                        .padding(PADDING),
                );
            } else if file_type.is_symlink() {
                let file_path_real = fs::read_link(&file_path).unwrap().to_path_buf();
                if let Ok(metadata) = fs::metadata(&file_path) {
                    if metadata.is_file() && is_json.is_match(file_path_real.to_str().unwrap()) {
                        let s = format!("{} -> {:?}", &file_name, &file_path_real);
                        col = col.push(
                            row![button(text(&s))
                                .style(iced::theme::Button::Custom(Box::new(GreenButton)))
                                .on_press(Message::LoadFile(file_path))]
                            .padding(PADDING),
                        );
                    } else if metadata.is_dir() {
                        let s = format!("{} -> {:?}", &file_name, &file_path_real);
                        col = col.push(
                            row![button(text(&s)).on_press(Message::ChangeDir(file_path))]
                                .padding(PADDING),
                        );
                    }
                } else {
                    col = col.push(row![text(&file_name)].padding(PADDING));
                }
            }
        }
        Ok(col)
    }

    pub fn show_hidden_files_toggle(&mut self) {
        self.show_hidden_files = !self.show_hidden_files;
    }
}

struct GreenButton;

impl button::StyleSheet for GreenButton {
    type Style = iced::Theme;

    fn active(&self, _: &<Self as button::StyleSheet>::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Color::from_rgb8(0, 255, 0).into()),
            ..Default::default()
        }
    }
}
