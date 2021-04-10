use tui_wrapper::*;
use window::window_layout_index;

use chrono::Local;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

fn draw_tab<B: Backend>(f: &mut Frame<B>, window: &Window) {
    f.render_widget(window.widget(), window.tab_chunk());
}

fn draw_panes<B: Backend>(f: &mut Frame<B>, tab: &Tab, selected_popup: bool) {
    for pane in tab.panes() {
        let selected = if selected_popup {
            false
        } else {
            pane.is_selected(tab.selected_pane())
        };

        let block = pane.block(selected);

        match pane.widget() {
            Widget::List(widget) => {
                f.render_stateful_widget(
                    widget.widget(block),
                    pane.chunk(),
                    &mut widget.state().borrow_mut(),
                );
            }
            Widget::Text(widget) => {
                f.render_widget(widget.widget().block(pane.block(selected)), pane.chunk());
            }
        }
    }
}

fn datetime() -> Span<'static> {
    Span::raw(format!(
        " {}",
        Local::now().format("%Y年%m月%d日 %H時%M分%S秒")
    ))
}

fn text_status((current, rows): (u64, u64)) -> Span<'static> {
    Span::raw(format!("{}/{}", current, rows))
}

fn scroll_status<'a>(window: &Window, id: &str) -> Option<Paragraph<'a>> {
    if let Some(pane) = window.selected_tab().panes().iter().find(|p| p.id() == id) {
        let widget = pane.widget().text();
        let span = match widget {
            Some(t) => text_status((t.selected(), t.row_size())),
            None => text_status((0, 0)),
        };

        let spans = Spans::from(span);
        let block = Block::default().style(Style::default());

        return Some(
            Paragraph::new(spans)
                .block(block)
                .alignment(Alignment::Right),
        );
    }
    None
}

fn draw_status<B: Backend>(f: &mut Frame<B>, chunk: Rect, window: &Window) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunk);

    let datetime = datetime();

    let datetime = Spans::from(datetime);
    let block = Block::default().style(Style::default());
    let paragraph = Paragraph::new(datetime).block(block);

    f.render_widget(paragraph, chunks[0]);

    if let Some(p) = scroll_status(&window, "logs") {
        f.render_widget(p, chunks[1]);
    }

    if let Some(p) = scroll_status(&window, "configs-raw") {
        f.render_widget(p, chunks[1]);
    }

    if let Some(p) = scroll_status(&window, "event") {
        f.render_widget(p, chunks[1]);
    }
}

fn draw_context<B: Backend>(f: &mut Frame<B>, chunk: Rect, ctx: &str, ns: &str) {
    let block = Block::default().style(Style::default());

    let text = format!("{}: {}", ns, ctx);
    let spans = Spans::from(text);
    let paragraph = Paragraph::new(spans).block(block);

    f.render_widget(paragraph, chunk);
}

pub fn draw<B: Backend>(f: &mut Frame<B>, window: &mut Window, ctx: &str, ns: &str) {
    let chunks = window.chunks();

    draw_tab(f, &window);

    draw_context(f, chunks[window_layout_index::CONTEXT], ctx, ns);

    draw_panes(f, window.selected_tab(), window.selected_popup());

    draw_status(f, chunks[window_layout_index::STATUSBAR], &window);

    if window.selected_popup() {
        let p = window.popup();
        let ns = p.widget().list().unwrap();
        f.render_widget(Clear, p.chunk());

        f.render_stateful_widget(
            ns.widget(p.block()),
            p.chunk(),
            &mut ns.state().borrow_mut(),
        );
    }
}
