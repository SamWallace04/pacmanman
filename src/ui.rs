use ratatui::{layout::*, style::palette::tailwind, style::*, text::*, widgets::*};

use crate::app::App;

const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;

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

pub fn render_menu<'a>(menu_titles: &Vec<&'a str>) -> Vec<Line<'a>> {
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

pub fn render_tabs<'a>(menu: Vec<Line<'a>>, active_menu_item: MenuItem) -> Tabs<'a> {
    Tabs::new(menu)
        .select(active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"))
}

pub fn render_footer<'a>() -> Paragraph<'a> {
    Paragraph::new("\nUse ↓/j and ↑/k to move, g/G to go top/bottom.").centered()
}

// TODO: Move the actual rendering into the functions.
pub fn render_package_table<'a>(app: &App) -> (List<'a>, Paragraph<'a>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Packages")
        .border_type(BorderType::Plain);

    let items: Vec<_> = app
        .list
        .packages
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![Span::styled(
                p.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let index = match app.list.state.selected() {
        Some(i) => i,
        None => 0,
    };

    let mut selected_package = app.list.packages[index].clone();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(SELECTED_STYLE_FG)
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
            "Depends On: ".to_owned()
                + package_details
                    .depends_on
                    .iter()
                    .map(|x| x.to_string() + ",")
                    .collect::<String>()
                    .trim_end_matches(","),
            Style::default(),
        ),
    ];

    let details_display = Paragraph::new(details_text).block(details_block);

    (list, details_display)
}
