use super::{tab::*, Pane, Popup};
use crate::widget::Widget;

use std::cell::RefCell;
use std::rc::Rc;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Tabs};

pub struct Window<'a> {
    tabs: Vec<Tab<'a>>,
    selected_tab_index: usize,
    layout: Layout,
    chunk: Rect,
}

// Tab
impl<'a> Window<'a> {
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
    pub fn pane(&self, id: impl Into<String>) -> Option<&Pane<'a>> {
        let id = id.into();
        for t in &self.tabs {
            return t.panes().iter().find(|p| p.id() == id);
        }
        None
    }
    pub fn pane_mut(&mut self, id: impl Into<String>) -> Option<&mut Pane<'a>> {
        let id = id.into();
        for t in &mut self.tabs {
            return t.panes_mut().iter_mut().find(|p| p.id() == id);
        }
        None
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
}

// widgetの状態変更
impl Window<'_> {
    pub fn select_next_item(&mut self) {
        self.selected_tab_mut().select_pane_next_item();
    }

    pub fn select_prev_item(&mut self) {
        self.selected_tab_mut().select_pane_prev_item();
    }

    pub fn select_first_item(&mut self) {
        self.selected_tab_mut().select_pane_first_item();
    }

    pub fn select_last_item(&mut self) {
        self.selected_tab_mut().select_pane_last_item();
    }

    pub fn scroll_up(&mut self) {
        let pane = self.selected_tab_mut().selected_pane_mut();
        let ch = pane.chunk();

        match pane.widget_mut() {
            Widget::List(list) => {
                list.prev();
            }
            Widget::Text(text) => {
                (0..ch.height).for_each(|_| text.prev());
            }
        }
    }

    pub fn scroll_down(&mut self) {
        let pane = self.selected_tab_mut().selected_pane_mut();
        let ch = pane.chunk();

        match pane.widget_mut() {
            Widget::List(list) => {
                list.next();
            }
            Widget::Text(text) => {
                (0..ch.height).for_each(|_| text.next());
            }
        }
    }
}

// Window
impl<'a> Window<'a> {
    pub fn new(tabs: Vec<Tab<'a>>) -> Self {
        Self {
            tabs,
            ..Window::default()
        }
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.chunk = chunk;

        let chunks = self.layout.split(chunk);

        self.tabs.iter_mut().for_each(|tab| {
            tab.update_chunk(chunks[1]);
            tab.update_popup_chunk(chunk);
        });
    }

    pub fn chunks(&self) -> Vec<Rect> {
        self.layout.split(self.chunk)
    }

    pub fn selected_pod(&self) -> String {
        let pane = self.selected_tab().selected_pane();
        let selected_index = pane
            .widget()
            .list()
            .unwrap()
            .state()
            .borrow()
            .selected()
            .unwrap();
        let split: Vec<&str> = pane.widget().list().unwrap().items()[selected_index]
            .split(' ')
            .collect();
        split[0].to_string()
    }

    pub fn widget(&self) -> Tabs {
        let titles: Vec<Spans> = self
            .tabs
            .iter()
            .map(|t| Spans::from(format!(" {} ", t.title())))
            .collect();

        let block = Block::default().style(Style::default());

        Tabs::new(titles)
            .block(block)
            .select(self.selected_tab_index)
            .highlight_style(Style::default().fg(Color::White).bg(Color::LightBlue))
    }

    pub fn tab_chunk(&self) -> Rect {
        self.chunks()[0]
    }
}

// 追い出したい
impl Window<'_> {
    pub fn log_status(&self) -> (u16, u16) {
        match self.selected_tab().selected_pane().widget().text() {
            Some(log) => (log.selected(), log.row_size()),
            None => (0, 0),
        }
    }

    pub fn update_wrap(&mut self) {
        let pane = self.pane_mut("logs");
        if let Some(p) = pane {
            let rect = p.chunk();
            let log = p.widget_mut().text_mut().unwrap();
            log.update_spans(rect.width);
            log.update_rows_size(rect.height);
        }
    }
}

// Popup
impl Window<'_> {
    pub fn popup(&self) -> Option<&Popup> {
        self.selected_tab().popup()
    }

    pub fn selected_popup(&self) -> bool {
        self.selected_tab().selected_popup()
    }

    pub fn select_popup(&mut self) {
        self.selected_tab_mut().select_popup();
    }
    pub fn unselect_popup(&mut self) {
        self.selected_tab_mut().unselect_popup();
    }
}

impl Default for Window<'_> {
    fn default() -> Self {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]
                .as_ref(),
            );

        Self {
            tabs: Vec::new(),
            selected_tab_index: 0,
            layout,
            chunk: Rect::default(),
        }
    }
}
