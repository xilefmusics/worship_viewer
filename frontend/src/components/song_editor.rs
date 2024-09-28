use super::{AspectRatio, SongViewer};
use fancy_yew::components::{Editor, SyntaxParser};
use shared::song::Song;
use std::f64::consts::SQRT_2;
use stylist::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub song: Song,
    pub onsave: Callback<Song>,
}

#[function_component(SongEditor)]
pub fn song_editor(props: &Props) -> Html {
    let onsave: Callback<String, ()> = {
        let onsave = props.onsave.clone();
        let id = props.song.id.clone();
        Callback::from(move |content: String| {
            let mut song = Song::try_from(content.as_str()).unwrap();
            song.id = id.clone();
            onsave.emit(song);
        })
    };

    let onautoformat = Callback::from(|content: String| {
        Song::try_from(content.as_str())
            .unwrap()
            .format_chord_pro(None, None)
    });

    let syntax_parser = SyntaxParser::builder()
        .transition("default", "{", "meta-begin", Some("default"), 1)
        .transition("meta-begin", "{", "meta-begin", None, 0)
        .transition("meta-begin", ":", "meta-middle", None, 1)
        .transition("meta-begin", "}", "meta-end", None, 1)
        .transition("meta-begin", "", "meta-key", Some("meta-surround"), 1)
        .transition("meta-key", "title:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", "artist:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", "key:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", "section:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", "language:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", ":", "meta-middle", Some("meta-key-error"), 1)
        .transition("meta-key", "}", "meta-end", Some("meta-key"), 1)
        .transition("meta-middle", ":", "meta-middle", None, 0)
        .transition("meta-middle", "}", "meta-end", None, 1)
        .transition("meta-middle", "", "meta-value", Some("meta-surround"), 1)
        .transition("meta-value", "}", "meta-end", Some("meta-value"), 1)
        .transition("meta-end", "}", "default", Some("meta-surround"), 0)
        .transition("default", "[", "chord", Some("default"), 1)
        .transition("chord", "[", "chord", None, 0)
        .transition("chord", "]", "default", Some("chord"), 0)
        .label_style("meta-surround", "font-weight", "bold")
        .label_style("meta-key", "color", "#cc241d")
        .label_style("meta-key-error", "text-decoration", "underline")
        .label_style("meta-key-error", "text-decoration-color", "#cc241d")
        .label_style("meta-value", "color", "#98971a")
        .label_style("chord", "color", "#d79921")
        .build()
        .expect("static parser should build");

    html! {
        <div class={Style::new(include_str!("song_editor.css")).expect("Unwrapping CSS should work!")}>
            <AspectRatio left={1./SQRT_2}>
                <SongViewer />
                <Editor
                    content={props.song.format_chord_pro(None, None)}
                    onsave={onsave}
                    onautoformat={onautoformat}
                    syntax_parser={syntax_parser}
                />
            </AspectRatio>
        </div>
    }
}
