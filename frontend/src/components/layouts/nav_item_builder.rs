use yew::prelude::*;
use yew_router::prelude::*;

pub struct NavItemBuilder<'a> {
    path: &'a str,
    icon: &'a str,
    text: &'a str,
    callback: Option<Callback<Navigator>>,
    index: bool,
}

impl<'a> NavItemBuilder<'a> {
    pub fn new() -> Self {
        Self {
            path: "",
            icon: "",
            text: "",
            callback: None,
            index: false,
        }
    }

    pub fn path(mut self, path: &'a str) -> Self {
        self.path = path;
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = icon;
        self
    }

    #[allow(dead_code)]
    pub fn text(mut self, text: &'a str) -> Self {
        self.text = text;
        self
    }

    pub fn callback(mut self, callback: Callback<Navigator>) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn index(mut self) -> Self {
        self.index = true;
        self
    }

    pub fn build(&self, navigator: &Navigator, current_path: &str) -> Html {
        let class = if current_path == self.path || current_path == "/" && self.index {
            "selected"
        } else {
            ""
        };

        let onclick = self.callback.clone().map(|callback| {
            let navigator = navigator.clone();
            move |_| callback.emit(navigator.clone())
        });

        html! {
            <li class={class} onclick={onclick}>
                <span class="material-symbols-outlined icon">{self.icon}</span>
                <span class="text">{self.text}</span>
            </li>
        }
    }
}
