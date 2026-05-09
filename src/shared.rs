use ratatui::{layout::*, style::*, widgets::*, Frame};

pub trait ListItems {}

pub struct StatefulList<T: ListItems> {
    pub state: ListState,
    pub items: Vec<T>,
    pub filtered_items: Vec<T>,
    pub last_selected: Option<usize>,
}

impl<T: ListItems> StatefulList<T> {
    pub fn new() -> Self {
        StatefulList {
            state: ListState::default(),
            items: vec![],
            last_selected: None,
            filtered_items: vec![],
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    pub fn go_top(&mut self) {
        self.state.select(Some(0));
    }

    pub fn go_bottom(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(self.filtered_items.len() - 1));
        }
    }
}

pub fn render_empty_list(frame: &mut Frame<'_>, chunk: Rect) {
    let para = Paragraph::new(
        "Could not find any packages that match the filter. Please reset it or try another.",
    )
    .style(Style::default());

    frame.render_widget(para, chunk);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

pub fn join_vec(vec: Vec<String>) -> String {
    vec.iter()
        .map(|x| x.to_string() + ",")
        .collect::<String>()
        .trim_end_matches(",")
        .to_string()
}
