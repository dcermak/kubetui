use crossbeam::channel::Sender;

use std::{cell::RefCell, rc::Rc};

use crate::{
    action::view_id,
    clipboard_wrapper::Clipboard,
    event::kubernetes::api_resources::ApiRequest,
    event::Event,
    tui_wrapper::{
        event::EventResult,
        tab::WidgetData,
        widget::{config::WidgetConfig, MultipleSelect, SelectedItem, Text, Widget, WidgetTrait},
        Tab, Window,
    },
};

pub struct ApiTabBuilder<'a> {
    title: &'a str,
    tx: &'a Sender<Event>,
    clipboard: &'a Option<Rc<RefCell<Clipboard>>>,
}

pub struct ApiTab {
    pub tab: Tab<'static>,
    pub popup: Widget<'static>,
}

impl<'a> ApiTabBuilder<'a> {
    pub fn new(
        title: &'static str,
        tx: &'a Sender<Event>,
        clipboard: &'a Option<Rc<RefCell<Clipboard>>>,
    ) -> Self {
        Self {
            title,
            tx,
            clipboard,
        }
    }

    pub fn build(self) -> ApiTab {
        let api = self.api();

        ApiTab {
            tab: Tab::new(view_id::tab_api, self.title, [WidgetData::new(api)]),
            popup: self.popup().into(),
        }
    }

    fn api(&self) -> Text {
        let tx = self.tx.clone();

        let open_subwin = move |w: &mut Window| {
            tx.send(ApiRequest::Get.into()).unwrap();
            w.open_popup(view_id::popup_api);
            EventResult::Nop
        };

        let builder = Text::builder()
            .id(view_id::tab_api_widget_api)
            .widget_config(&WidgetConfig::builder().title("API").build())
            .block_injection(|text: &Text, selected: bool| {
                let (index, size) = text.state();

                let mut config = text.widget_config().clone();

                *config.append_title_mut() = Some(format!(" [{}/{}]", index, size).into());

                config.render_block(text.focusable() && selected)
            })
            .action('f', open_subwin);

        if let Some(cb) = self.clipboard {
            builder.clipboard(cb.clone())
        } else {
            builder
        }
        .build()
    }

    fn popup(&self) -> MultipleSelect<'static> {
        let tx = self.tx.clone();

        MultipleSelect::builder()
            .id(view_id::popup_api)
            .widget_config(&WidgetConfig::builder().title("API").build())
            .on_select(move |w, _| {
                let widget = w
                    .find_widget_mut(view_id::popup_api)
                    .as_mut_multiple_select();

                if let Some(SelectedItem::Array(item)) = widget.widget_item() {
                    let apis = item.iter().map(|i| i.item.to_string()).collect();
                    tx.send(ApiRequest::Set(apis).into()).unwrap();
                }

                if widget.selected_items().is_empty() {
                    w.widget_clear(view_id::tab_api_widget_api)
                }

                EventResult::Nop
            })
            .build()
    }
}
