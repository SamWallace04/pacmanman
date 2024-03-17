use ratatui::{
    layout::*,
    style::{palette::tailwind, *},
    text::*,
    widgets::*,
    Frame,
};

use crate::app::App;

// TODO: Proper styling + config
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const SELECTED_STYLE_BG: Color = tailwind::BLACK;

pub enum UiEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    PackageList,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::PackageList => 0,
        }
    }
}

pub fn create_menu<'a>(menu_titles: &Vec<&'a str>) -> Vec<Line<'a>> {
    menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect()
}

pub fn render_tabs<'a>(
    menu: Vec<Line<'a>>,
    active_menu_item: MenuItem,
    frame: &mut Frame<'_>,
    chunk: Rect,
) {
    let tabs = Tabs::new(menu)
        .select(active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));

    frame.render_widget(tabs, chunk);
}

pub fn render_footer<'a>(frame: &mut Frame<'_>, chunk: Rect) {
    let footer = Paragraph::new("\nUse ↓/j and ↑/k to move, g/G to go top/bottom. i show explicitly installed packages, d show dependency packages, f to search, a to reset the filter").centered();
    frame.render_widget(footer, chunk);
}

impl App {
    pub fn render_package_table<'a>(&mut self, frame: &mut Frame<'_>, chunk: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
            .split(chunk);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Packages")
            .border_type(BorderType::Plain);

        let items: Vec<_> = self
            .packages_list
            .filtered_items
            .iter()
            .map(|p| {
                if p.is_dependency {
                    ListItem::new(Line::from(vec![Span::styled(
                        p.name.clone(),
                        Style::default().fg(Color::Black).bg(Color::Gray),
                    )]))
                } else {
                    ListItem::new(Line::from(vec![Span::styled(
                        p.name.clone(),
                        Style::default(),
                    )]))
                }
            })
            .collect();

        let index = match self.packages_list.state.selected() {
            Some(i) => i,
            None => 0,
        };

        let mut selected_package = self.packages_list.filtered_items[index].clone();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(SELECTED_STYLE_FG)
                    .bg(SELECTED_STYLE_BG)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        let package_details = selected_package.get_details();
        let details_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title(package_details.name + " Details")
            .border_type(BorderType::Plain);

        let details_text = vec![
            Line::styled(
                "Version: ".to_owned() + &selected_package.version.clone(),
                Style::default(),
            ),
            Line::styled(
                "Description: ".to_owned() + &package_details.description.clone(),
                Style::default(),
            ),
            Line::styled(
                "Depends On: ".to_owned() + &join_vec(package_details.depends_on.to_owned()),
                Style::default(),
            ),
            Line::styled(
                "Optional dependencies: ".to_owned()
                    + &join_vec(package_details.optional_dependencies).to_owned(),
                Style::default(),
            ),
            Line::styled(
                "Optional for: ".to_owned() + &join_vec(package_details.optional_for).to_owned(),
                Style::default(),
            ),
            Line::styled(
                "Installed size: ".to_owned() + &package_details.installed_size.clone(),
                Style::default(),
            ),
            Line::styled(
                "Install reason: ".to_owned() + &package_details.installed_reason.clone(),
                Style::default(),
            ),
        ];

        let details_display = Paragraph::new(details_text)
            .block(details_block)
            .wrap(Wrap { trim: false });

        frame.render_stateful_widget(list, layout[0], &mut self.packages_list.state);
        frame.render_widget(details_display, layout[1]);
    }
}

fn join_vec(vec: Vec<String>) -> String {
    vec.iter()
        .map(|x| x.to_string() + ",")
        .collect::<String>()
        .trim_end_matches(",")
        .to_string()
}
