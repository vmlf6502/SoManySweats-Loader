use crossterm::event::MouseEventKind;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use tui_big_text::BigText;
use tui_big_text::PixelSize;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

mod autostart;
use crate::autostart::*;
use crate::updater::Updater;
use crate::updater::VersionStatus;

mod updater;

fn main() -> io::Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        relaunch_in_terminal();
        return Ok(());
    }

    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
    ratatui::run(|terminal| App::default().run(terminal))?;
    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    Ok(())
}

#[derive(Debug)]
enum Status {
    Stopped,
    Started,
    Done,
    Error,
}

impl Status {
    fn as_str(&self) -> Span<'_> {
        match self {
            Status::Stopped => "Stopped".red(),
            Status::Started => "Started...".yellow(),
            Status::Done => "Done!".green(),
            Status::Error => "Error".red(),
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Default)]
pub struct App {
    exit: bool,
    log_reader: Option<BufReader<std::fs::File>>,
    output: Vec<String>,
    scroll: u16,
    status: Status,
    updater: Updater,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.updater = Updater::new();
        while !self.exit {
            self.handle_events()?;
            terminal.draw(|frame| self.render(frame))?;
            self.update_status();
        }
        Ok(())
    }

    fn update_status(&mut self) {
        if let Some(reader) = self.log_reader.as_mut() {
            // detect truncation
            let current_pos = reader.stream_position().unwrap();
            let file_len = reader.get_ref().metadata().unwrap().len();
            if file_len < current_pos {
                reader.seek(std::io::SeekFrom::Start(0)).unwrap();
            }

            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                if line.contains("LUNARCLIENT_STATUS_PREINIT") {
                    self.log_reader = None;
                    self.output.push(
                        "Lunar Client launch detected. Copying SoManySweats file into multiver..."
                            .into(),
                    );
                    self.copy_file();
                    return;
                }
                line.clear();
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_add(1),
                    MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_sub(1),
                    _ => {}
                },
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.toggle_command(),
            KeyCode::Char('a') => self.toggle_autostart(),
            KeyCode::Char('u') => self.update(),
            _ => {}
        }
    }

    fn update(&mut self) {
        match self.updater.status {
            VersionStatus::UpToDate => {}
            _ => match self.updater.update() {
                Ok(()) => {
                    self.output.push(format!(
                        "Successfully updated SoManySweats to v{}",
                        self.updater.release_ver
                    ));
                    self.updater.status = VersionStatus::UpToDate;
                }
                Err(e) => self
                    .output
                    .push(format!("Failed to update SoManySweats: {}", e)),
            },
        }
    }

    fn exit(&mut self) {
        self.stop();
        self.exit = true;
    }

    fn toggle_command(&mut self) {
        if self.status.as_str() == Status::Started.as_str() {
            self.stop();
        } else {
            self.start();
        }
    }

    fn start(&mut self) {
        self.status = Status::Started;
        let log = dirs::home_dir()
            .unwrap()
            .join(".lunarclient/profiles/1.8/logs/ichor-boot.log");
        let file = std::fs::File::open(log);
        match file {
            Ok(mut file) => {
                std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0)).unwrap();
                self.log_reader = Some(BufReader::new(file));
                self.output
                    .push("Started. Waiting for user to launch Lunar Client...".into());
            }
            Err(e) => {
                self.status = Status::Error;
                self.output.push(e.to_string());

                if let Some(os_error) = e.raw_os_error() {
                    if os_error == 2 {
                        self.output.push("Ensure that you launch Lunar Client with Forge at least once before using this loader.".into())
                    }
                }
            }
        }
    }

    fn stop(&mut self) {
        self.log_reader = None;
        self.status = Status::Stopped;
        self.output.push("Stopped".into());
    }

    fn copy_file(&mut self) {
        let base = dirs::home_dir()
            .unwrap()
            .join(".lunarclient/offline/multiver/somanysweats");
        let src = match std::fs::read_dir(&base) {
            Ok(src) => src,
            Err(_) => {
                self.output.push("Couldn't find the ~/.lunarclient/offline/multiver/somanysweats/ directory. Make sure you create one with that name and place the SoManySweats-vX.X.X.jar file in there.".into());
                self.status = Status::Error;
                return;
            }
        };

        let jar = match src
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_str().unwrap().starts_with("SoManySweats-"))
        {
            Some(jar) => jar,
            None => {
                self.output.push("Couldn't find SoManySweats-vX.X.X.jar in the ~/.lunarclient/offline/multiver/somanysweats/ directory. Make sure that file exists there.".into());
                self.status = Status::Error;
                return;
            }
        };

        let dst = base.join("ReplayMod-v1_8-2.6.14.jar");
        match std::fs::copy(jar.path(), dst) {
            Ok(_) => {
                self.status = Status::Done;
                self.output.push("Done!".into());
            }
            Err(e) => {
                self.output.push(format!("cp error: {e}"));
                self.status = Status::Error;
            }
        }
    }

    fn toggle_autostart(&mut self) {
        if auto_launch().is_enabled().unwrap_or(false) {
            auto_launch().disable().unwrap();
            self.output.push("AutoStart disabled".into());
        } else {
            auto_launch().enable().unwrap();
            self.output.push("AutoStart enabled".into());
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let autostart_enabled = auto_launch().is_enabled().unwrap_or(false);
        let needs_update = match self.updater.status {
            VersionStatus::UpToDate => false,
            _ => true,
        };

        let title = Line::from(vec![
            "SoManySweats Loader ".red(),
            "||".into(),
            format!(" SoManySweats-v{}", self.updater.current_ver).yellow(),
        ])
        .bold()
        .centered();

        let controls = Line::from(format!(
            "<A> AutoStart: {autostart_enabled} | <SPACE> Start/Stop | <Q> Quit"
        ));

        let update_text = Line::from(
            format!(
                "Update Available!   <U> Update/Install (v{} -> v{})",
                self.updater.current_ver, self.updater.release_ver
            )
            .green(),
        )
        .right_aligned();

        let vertical = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Length(1), // controls row
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
        let [title_area, controls_area, up, down] = frame.area().layout(&vertical);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(controls_area);

        frame.render_widget(title, title_area);
        frame.render_widget(Paragraph::new(controls), left_area);
        if needs_update {
            frame.render_widget(Paragraph::new(update_text), right_area);
        }
        frame.render_widget(self.draw_status(), up);
        frame.render_widget(self.draw_output(), down);
    }
    fn draw_status(&self) -> impl Widget {
        let title = Line::from(" Status ".bold());
        let block = Block::bordered()
            .border_set(border::THICK)
            .title(title.centered());

        BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![self.status.as_str().into()])
            .centered()
            .block(block)
            .build()
    }

    fn draw_output(&mut self) -> impl Widget {
        let title = Line::from(" Output ".bold());
        let block = Block::bordered()
            .border_set(border::THICK)
            .title(title.centered());
        let lines: Vec<Line> = self
            .output
            .iter()
            .map(|s| Line::from(format!(" >> {}", s.as_str())).light_blue())
            .collect();

        Paragraph::new(lines).block(block).scroll((self.scroll, 0))
    }
}
