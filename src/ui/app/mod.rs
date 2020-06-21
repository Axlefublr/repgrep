mod item;

use std::collections::VecDeque;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use either::Either;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, List, ListState, Paragraph, Text};
use tui::Frame;

use crate::cli::Args;
use crate::rg::de::{RgMessageType, Stats};
use item::{Item, ItemKind};

fn clamp(val: usize, min: usize, max: usize) -> usize {
  if val <= min {
    min
  } else if val >= max {
    max
  } else {
    val
  }
}

#[derive(Debug, Eq, PartialEq)]
enum Movement {
  Prev,
  Next,
  PrevFile,
  NextFile,
  Forward(u16),
  Backward(u16),
}

#[derive(Debug, Eq, PartialEq)]
enum AppState {
  SelectMatches,
  InputReplacement(String),
}

impl AppState {
  pub fn to_text(&self) -> Text {
    match self {
      AppState::SelectMatches => Text::styled(
        " SELECT ",
        Style::default().bg(Color::Cyan).fg(Color::Black),
      ),
      AppState::InputReplacement(_) => Text::styled(
        " REPLACE ",
        Style::default().bg(Color::White).fg(Color::Black),
      ),
    }
  }
}

pub struct App {
  pub should_quit: bool,

  rg_cmdline: String,
  stats: Stats,
  list: Vec<Item>,
  list_state: ListState,
  state: AppState,
}

// General impl.
impl App {
  pub fn new(args: &Args, mut rg_results: VecDeque<RgMessageType>) -> App {
    let mut list = vec![];
    let mut maybe_stats = None;
    while let Some(rg_type) = rg_results.pop_front() {
      match rg_type {
        RgMessageType::Summary { stats, .. } => {
          maybe_stats = Some(stats);
          // NOTE: there should only be one RgMessageType::Summary, and it should be the last item.
          break;
        }
        t => list.push(Item::new(t)),
      }
    }

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    App {
      rg_cmdline: format!("rg {}", args.rg_args.join(" ")),
      should_quit: false,
      stats: maybe_stats.expect("failed to find RgMessageType::Summary from rg!"),
      list_state,
      list,
      state: AppState::SelectMatches,
    }
  }
}

// Rendering.
impl App {
  // The UI is:
  // _
  // | - list
  // | - of
  // | - matches
  // | repgrep status line (rg cmdline, matches, replacements, etc)
  // | repgrep command line (user input for replacement text, etc)
  // _
  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let (root_split, stats_and_input_split) = self.get_layouts(f.size());
    self.draw_match_list(f, root_split[0]);
    self.draw_stats_line(f, stats_and_input_split[0]);
    self.draw_input_line(f, stats_and_input_split[1]);
  }

  fn get_layouts(&self, r: Rect) -> (Vec<Rect>, Vec<Rect>) {
    let root_split = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
      .split(r);

    let stats_and_input_split = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Length(1)].as_ref())
      .split(root_split[1]);

    (root_split, stats_and_input_split)
  }

  fn draw_input_line<B: Backend>(&mut self, f: &mut Frame<B>, r: Rect) {
    let text_items = match &self.state {
      AppState::SelectMatches => vec![Text::raw(
        "Select (or deselect) Matches with <space> then press <Enter>...",
      )],
      AppState::InputReplacement(input) => vec![
        Text::raw("Replacement: "),
        if input.is_empty() {
          Text::styled("<Enter Replacement>", Style::default().fg(Color::DarkGray))
        } else {
          Text::raw(input)
        },
      ],
    };
    f.render_widget(Paragraph::new(text_items.iter()), r);
  }

  fn draw_stats_line<B: Backend>(&mut self, f: &mut Frame<B>, r: Rect) {
    let replacement_count = self
      .list
      .iter()
      .filter_map(|i| {
        if matches!(i.kind, ItemKind::Match) && i.should_replace {
          Some(i.match_count())
        } else {
          None
        }
      })
      .sum::<usize>();

    // Split the stats line into halves.
    let hsplit = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Length(10), Constraint::Min(1)].as_ref())
      .split(r);

    let left_side_items = [self.state.to_text()];
    let right_side_items = [
      Text::styled(
        format!(" {} ", self.rg_cmdline),
        Style::default().bg(Color::Blue).fg(Color::Black),
      ),
      Text::styled(
        format!(" Matches: {} ", self.stats.matches),
        Style::default().bg(Color::Cyan).fg(Color::Black),
      ),
      Text::styled(
        format!(" Replacements: {} ", replacement_count),
        Style::default().bg(Color::Magenta).fg(Color::Black),
      ),
    ];

    let stats_line_style = Style::default().bg(Color::DarkGray).fg(Color::White);
    f.render_widget(
      Paragraph::new(left_side_items.iter())
        .style(stats_line_style)
        .alignment(Alignment::Left),
      hsplit[0],
    );
    f.render_widget(
      Paragraph::new(right_side_items.iter())
        .style(stats_line_style)
        .alignment(Alignment::Right),
      hsplit[1],
    );
  }

  fn draw_match_list<B: Backend>(&mut self, f: &mut Frame<B>, r: Rect) {
    let match_items = self.list.iter().map(|item| item.to_text());

    let curr_item = &self.list[self.curr_pos()];
    let highlight_style = Style::default().fg(if matches!(curr_item.kind, ItemKind::Match) {
      if curr_item.should_replace {
        Color::Yellow
      } else {
        Color::Red
      }
    } else if matches!(curr_item.kind, ItemKind::Begin) {
      Color::Magenta
    } else {
      Color::DarkGray
    });

    // TODO: highlight the whole line (not just the text on it), currently not possible
    // See: https://github.com/fdehau/tui-rs/issues/239
    let match_list = List::new(match_items)
      .block(Block::default())
      .style(Style::default().fg(Color::White))
      .highlight_symbol("-> ")
      .highlight_style(highlight_style);

    f.render_stateful_widget(match_list, r, &mut self.list_state)
  }
}

// Event Handling.
impl App {
  fn list_height(&self, term_size: Rect) -> u16 {
    let (root_split, _) = self.get_layouts(term_size);
    root_split[0].height
  }

  pub fn on_event(&mut self, term_size: Rect, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
      match self.state {
        AppState::SelectMatches => {
          // CONTROL+KEY
          if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
              KeyCode::Char('b') => self.move_pos(Movement::Backward(self.list_height(term_size))),
              KeyCode::Char('f') => self.move_pos(Movement::Forward(self.list_height(term_size))),
              _ => {}
            }
          } else {
            match key.code {
              KeyCode::Up | KeyCode::Char('k') => self.move_pos(Movement::Prev),
              KeyCode::Down | KeyCode::Char('j') => self.move_pos(Movement::Next),
              KeyCode::Char('K') => self.move_pos(Movement::PrevFile),
              KeyCode::Char('J') => self.move_pos(Movement::NextFile),
              KeyCode::Char(' ') => self.toggle_item(),
              KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
              KeyCode::Enter => self.state = AppState::InputReplacement(String::new()),
              _ => {}
            }
          }
        }
        AppState::InputReplacement(ref input) => {
          match key.code {
            KeyCode::Char(c) => {
              let mut new_input = String::from(input);
              new_input.push(c);
              self.state = AppState::InputReplacement(new_input);
            }
            KeyCode::Backspace | KeyCode::Delete => {
              let new_input = if !input.is_empty() {
                String::from(input)[..input.len() - 1].to_owned()
              } else {
                String::new()
              };
              self.state = AppState::InputReplacement(new_input);
            }
            KeyCode::Esc => self.state = AppState::SelectMatches,
            KeyCode::Enter => {
              // TODO: confirm replacement before continuing
              // TODO: perform replacements and quit
            }
            _ => {}
          }
        }
      }
    }

    Ok(())
  }

  fn curr_pos(&self) -> usize {
    self.list_state.selected().unwrap_or(0)
  }

  // TODO: support selecting submatches
  fn move_pos(&mut self, direction: Movement) {
    let iterator = self.list.iter().enumerate();
    let iterator = match direction {
      Movement::Prev | Movement::PrevFile | Movement::Backward(_) => Either::Left(iterator.rev()),
      Movement::Next | Movement::NextFile | Movement::Forward(_) => Either::Right(iterator),
    };

    let current = self.curr_pos();
    let (skip, default) = match direction {
      Movement::Prev | Movement::PrevFile => (self.list.len().saturating_sub(current), 0),
      Movement::Backward(n) => (
        self
          .list
          .len()
          .saturating_sub(current.saturating_sub(n as usize)),
        0,
      ),

      Movement::Next | Movement::NextFile => (current, self.list.len() - 1),
      Movement::Forward(n) => (current + (n as usize), self.list.len() - 1),
    };

    let pos = iterator
      .skip(skip)
      .find_map(|(i, item)| {
        let is_valid_next = match direction {
          Movement::PrevFile => i < current && matches!(item.kind, ItemKind::Begin),
          Movement::NextFile => i > current && matches!(item.kind, ItemKind::Begin),
          Movement::Prev | Movement::Backward(_) => i < current,
          Movement::Next | Movement::Forward(_) => i > current,
        };

        if is_valid_next && item.is_selectable() {
          Some(i)
        } else {
          None
        }
      })
      .unwrap_or(default);

    self
      .list_state
      .select(Some(clamp(pos, 0, self.list.len() - 1)));
  }

  fn toggle_item(&mut self) {
    let curr_pos = self.curr_pos();

    // If Match item, toggle replace.
    if matches!(self.list[curr_pos].kind, ItemKind::Match) {
      let selected_item = &mut self.list[curr_pos];
      selected_item.should_replace = !selected_item.should_replace;
    }

    // If Begin item, toggle all matches in it.
    if matches!(self.list[curr_pos].kind, ItemKind::Begin) {
      let mut items_to_toggle: Vec<_> = self
        .list
        .iter_mut()
        .skip(curr_pos)
        .take_while(|i| i.kind != ItemKind::End)
        .filter(|i| i.kind == ItemKind::Match)
        .collect();

      let should_replace = items_to_toggle.iter().all(|i| !i.should_replace);
      for item in items_to_toggle.iter_mut() {
        item.should_replace = should_replace;
      }
    }
  }
}