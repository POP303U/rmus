use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::{
    env::{self},
    io::{self, stdout, BufReader},
    thread::sleep,
};

use ratatui::{prelude::*, widgets::*};
use rodio::{OutputStream, Sink};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rmus <input>");
        std::process::exit(1);
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create an output stream
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Create a sink
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Load an audio file
    let file_path = &args[1];
    let file = std::fs::File::open(file_path).expect("Failed to open file");
    let source = rodio::Decoder::new(BufReader::new(file)).expect("Failed to create decoder");

    // Add the audio source to the sink
    sink.append(source);

    // Play the audio
    sink.play();

    let mut should_quit = false;
    let mut is_paused = false;
    while !should_quit {
        terminal.draw(render_ui)?;
        if !sink.empty() && !is_paused {
            sleep(std::time::Duration::from_millis(100));
        }
        should_quit = match handle_events(&sink, &mut is_paused) {
            Ok(value) => value,
            Err(_) => false,
        };
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events(sink: &Sink, is_paused: &mut bool) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(1))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('c') => handle_event_pause(&sink, is_paused),
                KeyCode::Char('-') => handle_event_volume_down(&sink),
                KeyCode::Char('+') => handle_event_volume_up(&sink),
                _ => {}
            }
        }
    }
    Ok(false)
}

fn handle_event_volume_down(sink: &Sink) {
    let current_volume = sink.volume();
    if current_volume > 0.1 {
        sink.set_volume(current_volume - 0.1);
    }
}

fn handle_event_volume_up(sink: &Sink) {
    let current_volume = sink.volume();
    if current_volume < 0.9 {
        sink.set_volume(current_volume + 0.1);
    }
}

fn handle_event_pause(sink: &Sink, is_paused: &mut bool) {
    if *is_paused {
        sink.play();
    } else {
        sink.pause();
    }
    *is_paused = !*is_paused;
}

fn render_ui(frame: &mut Frame) {
    let args: Vec<String> = env::args().collect();
    frame.render_widget(
        Paragraph::new(&*args[1]).block(Block::default().title("Song").borders(Borders::ALL)),
        frame.size(),
    );
}
