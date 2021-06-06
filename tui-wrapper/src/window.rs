use chrono::Local;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Clear, Paragraph, Tabs},
    Frame,
};

use unicode_width::UnicodeWidthStr;

use crate::util::child_window_chunk;

use super::{
    event::{EventResult, InnerCallback},
    util,
    widget::{RenderTrait, Widget, WidgetTrait},
    Tab,
};

use ::event::UserEvent;

#[derive(Default)]
pub struct Window<'a> {
    tabs: Vec<Tab<'a>>,
    selected_tab_index: usize,
    layout: Layout,
    chunk: Rect,
    status_target_id: Vec<(&'a str, &'a str)>,
    callbacks: Vec<(UserEvent, InnerCallback)>,
    popup: Vec<Widget<'a>>,
    open_popup_id: Option<String>,
}

pub mod window_layout_index {
    pub const TAB: usize = 0;
    pub const CONTEXT: usize = 2;
    pub const CONTENTS: usize = 3;
    pub const STATUSBAR: usize = 4;
}

// Window
impl<'a> Window<'a> {
    pub fn new(tabs: Vec<Tab<'a>>) -> Self {
        Self {
            tabs,
            selected_tab_index: 0,
            layout: Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                ),
            ..Default::default()
        }
    }

    pub fn add_popup(&mut self, popup: Vec<Widget<'a>>) {
        self.popup = popup;
    }

    pub fn open_popup(&mut self, id: impl Into<String>) {
        self.open_popup_id = Some(id.into());
    }

    pub fn close_popup(&mut self) {
        self.open_popup_id = None;
    }

    pub fn opening_popup(&self) -> bool {
        self.open_popup_id.is_some()
    }

    pub fn popup(&self, id: &str) -> Option<&Widget<'a>> {
        self.popup.iter().find(|w| w.id() == id)
    }

    pub fn popup_mut(&mut self, id: &str) -> Option<&mut Widget<'a>> {
        self.popup.iter_mut().find(|w| w.id() == id)
    }

    pub fn status_target_id(mut self, id: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
        self.status_target_id = id.into();
        self
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.chunk = chunk;

        let chunks = self.layout.split(chunk);

        self.tabs.iter_mut().for_each(|tab| {
            tab.update_chunk(chunks[window_layout_index::CONTENTS]);
        });

        self.popup.iter_mut().for_each(|w| {
            w.update_chunk(util::default_focus_block().inner(child_window_chunk(80, 80, chunk)))
        })
    }

    pub fn chunks(&self) -> Vec<Rect> {
        self.layout.split(self.chunk)
    }

    pub fn widget(&self) -> Tabs {
        let titles: Vec<Spans> = self
            .tabs
            .iter()
            .map(|t| Spans::from(Self::tab_title_format(t.title())))
            .collect();

        Tabs::new(titles)
            .block(Self::tab_block())
            .select(self.selected_tab_index)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    }

    fn tab_title_format(title: &str) -> String {
        format!(" {} ", title)
    }

    fn tab_block() -> Block<'a> {
        Block::default().style(Style::default())
    }

    pub fn tab_chunk(&self) -> Rect {
        self.chunks()[window_layout_index::TAB]
    }

    pub fn match_callback(&self, ev: UserEvent) -> Option<InnerCallback> {
        self.callbacks
            .iter()
            .find_map(|(cb_ev, cb)| if *cb_ev == ev { Some(cb.clone()) } else { None })
    }
}

// Tab
impl<'a> Window<'a> {
    pub fn selected_tab_id(&self) -> &str {
        &self.tabs[self.selected_tab_index].id()
    }

    pub fn selected_tab(&self) -> &Tab {
        &self.tabs[self.selected_tab_index]
    }

    pub fn selected_tab_mut(&mut self) -> &mut Tab<'a> {
        &mut self.tabs[self.selected_tab_index]
    }

    pub fn select_tab(&mut self, index: usize) {
        let index = index - 1;
        if index < self.tabs.len() {
            self.selected_tab_index = index;
        }
    }

    pub fn select_next_tab(&mut self) {
        if self.tabs.len() - 1 <= self.selected_tab_index {
            self.selected_tab_index = 0;
        } else {
            self.selected_tab_index += 1;
        }
    }

    pub fn select_prev_tab(&mut self) {
        if 0 == self.selected_tab_index {
            self.selected_tab_index = self.tabs.len() - 1;
        } else {
            self.selected_tab_index -= 1;
        }
    }
}

// Pane
impl<'a> Window<'a> {
    pub fn pane(&self, id: &str) -> Option<&Widget<'a>> {
        for t in &self.tabs {
            let w = t.panes().into_iter().find(|p| p.id() == id);
            if w.is_some() {
                return w;
            }
        }

        self.popup(id)
    }
    pub fn pane_mut(&mut self, id: &str) -> Option<&mut Widget<'a>> {
        if self.opening_popup() {
            let w = self.popup.iter_mut().find(|w| w.id() == id);

            if w.is_some() {
                w
            } else {
                self.tabs
                    .iter_mut()
                    .find_map(|t| t.panes_mut().into_iter().find(|p| p.id() == id))
            }
        } else {
            let w = self
                .tabs
                .iter_mut()
                .find_map(|t| t.panes_mut().into_iter().find(|p| p.id() == id));

            if w.is_some() {
                w
            } else {
                self.popup.iter_mut().find(|w| w.id() == id)
            }
        }
    }

    pub fn selected_pane_id(&self) -> &str {
        self.selected_tab().selected_pane_id()
    }

    pub fn select_next_pane(&mut self) {
        self.selected_tab_mut().next_pane();
    }

    pub fn select_prev_pane(&mut self) {
        self.selected_tab_mut().prev_pane();
    }

    pub fn pane_clear(&mut self, id: &str) {
        if let Some(pane) = self.pane_mut(id) {
            pane.clear();
        }
    }

    pub fn select_pane(&mut self, id: &str) {
        self.selected_tab_mut().select_pane(id)
    }
}

// Render
use window_layout_index::*;
impl<'a> Window<'a> {
    pub fn render<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        current_context: &str,
        current_namespace: &str,
    ) {
        self.render_tab(f);

        self.render_context(f, current_context, current_namespace);

        self.selected_tab_mut().render(f);

        self.render_status(f);

        if let Some(id) = &self.open_popup_id {
            if let Some(popup) = self.popup.iter_mut().find(|p| p.id() == id) {
                f.render_widget(Clear, child_window_chunk(80, 80, self.chunk));
                popup.render(f, true);
            }
        }
    }

    fn render_tab<B: Backend>(&mut self, f: &mut Frame<B>) {
        f.render_widget(self.widget(), self.tab_chunk());
    }

    fn render_context<B: Backend>(&mut self, f: &mut Frame<B>, ctx: &str, ns: &str) {
        let block = Block::default().style(Style::default());

        let text = format!("{}: {}", ns, ctx);
        let spans = Spans::from(text);
        let paragraph = Paragraph::new(spans).block(block);

        f.render_widget(paragraph, self.chunks()[CONTEXT]);
    }

    fn render_status<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(28), Constraint::Min(1)])
            .split(self.chunks()[STATUSBAR]);

        let datetime = datetime();

        let datetime = Spans::from(datetime);
        let block = Block::default().style(Style::default());
        let paragraph = Paragraph::new(datetime).block(block);

        f.render_widget(paragraph, chunks[0]);

        let widget: Option<Paragraph> = if let Some(id) = self
            .status_target_id
            .iter()
            .find(|id| id.0 == self.selected_tab_id())
        {
            self.scroll_status(id.1)
        } else {
            None
        };

        if let Some(widget) = widget {
            f.render_widget(widget.alignment(Alignment::Right), chunks[1]);
        }
    }

    fn scroll_status(&self, id: &str) -> Option<Paragraph> {
        if let Some(pane) = self.selected_tab().panes().iter().find(|p| p.id() == id) {
            let widget = pane.as_text();

            let spans = widget.status();
            let block = Block::default().style(Style::default());

            return Some(Paragraph::new(spans).block(block));
        }
        None
    }
}

fn datetime() -> Span<'static> {
    Span::raw(format!(
        " {}",
        Local::now().format("%Y年%m月%d日 %H時%M分%S秒")
    ))
}

pub enum WindowEvent {
    CloseWindow,
    Continue,
    UpdateContents(::event::kubernetes::Kube),
    ResizeWindow(u16, u16),
}

// Event
impl Window<'_> {
    pub fn add_action<F, E: Into<UserEvent>>(&mut self, ev: E, cb: F)
    where
        F: Fn(&mut Window) -> EventResult + 'static,
    {
        self.callbacks.push((ev.into(), Rc::new(cb)));
    }

    pub fn on_event(&mut self, ev: UserEvent) -> EventResult {
        match ev {
            UserEvent::Key(ev) => self.on_key_event(ev),
            UserEvent::Mouse(ev) => self.on_mouse_event(ev),
            UserEvent::Resize(w, h) => EventResult::Window(WindowEvent::ResizeWindow(w, h)),
        }
    }

    pub fn on_key_event(&mut self, ev: KeyEvent) -> EventResult {
        if let Some(id) = &self.open_popup_id {
            if let Some(popup) = self.popup.iter_mut().find(|w| w.id() == id) {
                return popup.on_key_event(ev);
            }
        }

        let focus_pane = self.selected_tab_mut().selected_pane_mut();

        match focus_pane.on_key_event(ev) {
            EventResult::Ignore => match util::key_event_to_code(ev) {
                KeyCode::Tab => {
                    self.select_next_pane();
                }

                KeyCode::BackTab => {
                    self.select_prev_pane();
                }

                KeyCode::Char(n @ '1'..='9') => {
                    self.select_tab(n as usize - b'0' as usize);
                }

                _ => {
                    return EventResult::Ignore;
                }
            },
            ev => {
                return ev;
            }
        }

        EventResult::Nop
    }

    pub fn on_mouse_event(&mut self, ev: MouseEvent) -> EventResult {
        if let Some(id) = &self.open_popup_id {
            if let Some(popup) = self.popup.iter_mut().find(|w| w.id() == id) {
                return popup.on_mouse_event(ev);
            }
        }

        let pos = (ev.column, ev.row);
        let selected_pane_id = self.selected_pane_id().to_string();
        let mut focus_pane_id = None;

        let result = if util::contains(self.tab_chunk(), pos) {
            self.on_click_tab(ev);
            EventResult::Nop
        } else if util::contains(self.chunks()[window_layout_index::CONTENTS], pos) {
            if let Some(pane) = self
                .selected_tab_mut()
                .panes_mut()
                .iter_mut()
                .find(|pane| util::contains(pane.chunk(), pos))
            {
                focus_pane_id = if pane.id() != selected_pane_id {
                    Some(pane.id().to_string())
                } else {
                    None
                };
                pane.on_mouse_event(ev)
            } else {
                EventResult::Ignore
            }
        } else {
            EventResult::Ignore
        };

        if let Some(id) = focus_pane_id {
            self.select_pane(&id);
        }

        result
    }

    fn on_click_tab(&mut self, ev: MouseEvent) {
        if ev.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }

        let pos = util::mouse_pos(ev);

        let chunk = Self::tab_block().inner(self.tab_chunk());
        let divider_width = 1;

        let mut x = chunk.left();
        let y = chunk.top();
        let h = chunk.height;

        for (i, tab) in self.tabs.iter().enumerate() {
            let w = Self::tab_title_format(tab.title()).width() as u16;
            x = x.saturating_add(1);

            let title_chunk = Rect::new(x, y, w, h);

            if util::contains(title_chunk, pos) {
                self.select_tab(i + 1);
                break;
            }

            x = x
                .saturating_add(1)
                .saturating_add(w)
                .saturating_add(divider_width);
        }
    }
}
