use super::{AspectRatio, SongViewer};
use fancy_yew::components::input::StringInput;
use fancy_yew::components::{Editor, SyntaxParser};
use shared::song::Song;
use fancy_yew::toast_notifications::show_error;
use std::f64::consts::SQRT_2;
use stylist::Style;
use yew::prelude::*;
use yew_hooks::use_size;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub song: Song,
    pub onsave: Callback<Song>,
    pub onback: Callback<MouseEvent>,
    pub onimport: Callback<String>,
}

#[function_component(SongEditor)]
pub fn song_editor(props: &Props) -> Html {
    let div_ref = use_node_ref();
    let state = use_size(div_ref.clone());
    let show_viewer = state.0 > state.1;
    let width = if show_viewer {
        state.0 - (state.1 as f64 / SQRT_2) as u32
    } else {
        state.0
    };

    let new = use_state(|| false);
    let toggle_new = {
        let new = new.clone();
        move |_: MouseEvent| {
            new.set(!*new);
        }
    };
    let import_url = use_state(|| String::default());

    let onsave: Callback<String, ()> = {
        let onsave = props.onsave.clone();
        let id = props.song.id.clone();
        Callback::from(move |content: String| match Song::try_from(content.as_str()) {
            Ok(mut song) => {
                if song.id.is_none() {
                    song.id = id.clone();
                }
                onsave.emit(song);
            }
            Err(e) => {
                show_error(
                    "Error parsing song",
                    &format!("{e}"),
                );
            }
        })
    };

    let onautoformat = Callback::from(|content: String| match Song::try_from(content.as_str()) {
        Ok(song) => song.format_worship_pro(None, None),
        Err(e) => {
            show_error(
                "Error parsing song",
                &format!("{e}"),
            );
            content
        }
    });

    let onimport = {
        let import_url = import_url.clone();
        let new = new.clone();
        let onimport = props.onimport.clone();
        move |_: MouseEvent| {
            onimport.emit((*import_url).clone());
            new.set(false);
        }
    };

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
        .transition("meta-key", "tempo:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", "time:", "meta-middle", Some("meta-key"), 1)
        .transition("meta-key", "comment:", "meta-middle", Some("meta-key"), 1)
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

    let editor_html = html! {
        { if *new {html!{
            <div class="editor-new">
                <StringInput
                    bind_handle={import_url}
                    placeholder={"Enter a URL to https://tabs.ultimate-guitar.com/ or leave empty".to_string()}

                />
                <button class="editor-new-button" onclick={onimport}>
                    {"Import or Create"}
                </button>
            </div>
        }} else {html!{
            <div class="editor-wrapper">
                <Editor
                    content={props.song.format_worship_pro(None, None)}
                    onsave={onsave}
                    onautoformat={onautoformat}
                    syntax_parser={syntax_parser}
                    style={format!("width: {}px;", width)}
                />
            </div>
        }}}
    };

    html! {
        <div class={Style::new(include_str!("song_editor.css")).expect("Unwrapping CSS should work!")}>
            <div class="editor-header">
                <span
                    class="material-symbols-outlined button"
                    onclick={props.onback.clone()}
                >{"arrow_back"}</span>
                <div class="seperator"></div>
                <span
                    class="material-symbols-outlined button"
                    onclick={toggle_new}
                >{"add"}</span>
            </div>
            <div ref={div_ref} class="editor-main">
                { if show_viewer {html!{
                    <AspectRatio left={1./SQRT_2}>
                        <SongViewer
                            song={props.song.clone()}
                        />
                        {editor_html}
                    </AspectRatio>
                }} else {html!{
                    {editor_html}
                }}}
            </div>
        </div>
    }
}
