use super::{Slide, SongData, SettingsData};
use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SlidesProps {
    pub data: SongData,
    pub set_current: Callback<(usize, usize)>,
    pub settings: SettingsData,
}

#[function_component(Slides)]
pub fn slides(props: &SlidesProps) -> Html {
    html! {
        <div class={Style::new(include_str!("slides.css")).expect("Unwrapping CSS should work!")}>
        {
            for props.data.outline.iter().filter(|outline| !outline.duplicate && outline.has_text).flat_map(|outline| {
                let title = outline.title.clone();
                (0..outline.len).map(move |offset| {
                    let slide_idx = outline.text_idx + offset;
                    let text = props.data.slides.get(slide_idx).cloned().unwrap_or(String::new());

                    let onclick = {
                        let set_current = props.set_current.clone();
                        Callback::from(move |_| {
                            set_current.emit((slide_idx, usize::MAX));
                        })
                    };

                    html! {
                        <div class="slide-wrapper" {onclick}>
                            <div class="slide-header">{ if offset == 0 { title.clone() } else { format!("{} ({})", title, offset+1) } }</div>
                            <Slide
                                text={text}
                                settings={props.settings.clone()}
                            />
                        </div>
                    }
                })
            })
        }
        </div>
    }
}
