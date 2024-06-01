use stylist::Style;
use worship_viewer_shared::player::PlayerItem;
use worship_viewer_shared::song::{FormatOutputLines, OutputLine};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub item: PlayerItem,
    pub font_size: i32,
}

#[function_component]
pub fn PageComponent(props: &Props) -> Html {
    match &props.item {
        PlayerItem::Blob(id) => html! {
            <div class={Style::new(include_str!("page.css")).expect("Unwrapping CSS should work!")}>
                <img src={format!("/api/blobs/{}", id)}/>
            </div>
        },
        PlayerItem::Chords(song) => {
            html! {
                <div
                    style={format!("font-size: {}px", &props.font_size)}
                    class={Style::new(include_str!("page.css")).expect("Unwrapping CSS should work!")}
                >
                    <div class="wrapper">
                        <div class="title">{format!("{} ({})", &song.title, &song.artist)}</div>
                        {song.format_output_lines(None, None).iter().map(|line| match line {
                            OutputLine::Keyword(text) => html!{<span class="keyword">{text}</span>},
                            OutputLine::Chord(text) => html!{<span class="chord">{text}</span>},
                            OutputLine::Text(text) => html!{<span class="text">{text}</span>},
                        }).collect::<Html>()}
                    </div>
                </div>
            }
        }
    }
}
