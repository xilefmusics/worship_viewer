use super::{SlideTextOrientation, HorizontalContainerAlignment, TextAlignment};
use stylist::Style;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Properties, Serialize, Deserialize, PartialEq, Clone)]
pub struct SettingsData {
    pub max_lines_per_slide: u8,
    pub background: u8,
    pub text_orientation: SlideTextOrientation,
    pub font_size: u8,
    pub horizontal_container_alignment: HorizontalContainerAlignment,
    pub text_alignment: TextAlignment,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            max_lines_per_slide: 2,
            background: 2,
            text_orientation: SlideTextOrientation::Center,
            font_size: 60,
            horizontal_container_alignment: HorizontalContainerAlignment::Center,
            text_alignment: TextAlignment::Center,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct SettingsProps {
    pub settings: SettingsData,
    pub set_settings: Callback<SettingsData>,
}

#[function_component(Settings)]
pub fn settings(props: &SettingsProps) -> Html {
    let set_max_lines_per_slide = {
        let settings = props.settings.clone();
        let set_settings = props.set_settings.clone();

        Callback::from(move |num: u8| {
            let mut settings = settings.clone();
            settings.max_lines_per_slide = num;
            set_settings.emit(settings);
        })
    };

    let set_background = {
        let settings = props.settings.clone();
        let set_settings = props.set_settings.clone();

        Callback::from(move |num: u8| {
            let mut settings = settings.clone();
            settings.background = num;
            set_settings.emit(settings);
        })
    };

    let set_text_orientation = {
        let settings = props.settings.clone();
        let set_settings = props.set_settings.clone();

        Callback::from(move |orientation: SlideTextOrientation| {
            let mut settings = settings.clone();
            settings.text_orientation = orientation;
            set_settings.emit(settings);
        })
    };

    let set_font_size = {
        let settings = props.settings.clone();
        let set_settings = props.set_settings.clone();

        Callback::from(move |num: u8| {
            let mut settings = settings.clone();
            settings.font_size = num;
            set_settings.emit(settings);
        })
    };

    let set_horizontal_container_alignment = {
        let settings = props.settings.clone();
        let set_settings = props.set_settings.clone();

        Callback::from(move |alignment: HorizontalContainerAlignment| {
            let mut settings = settings.clone();
            settings.horizontal_container_alignment = alignment;
            set_settings.emit(settings);
        })
    };

    let set_text_alignment = {
        let settings = props.settings.clone();
        let set_settings = props.set_settings.clone();

        Callback::from(move |alignment: TextAlignment| {
            let mut settings = settings.clone();
            settings.text_alignment = alignment;
            set_settings.emit(settings);
        })
    };

    html! {
        <div class={Style::new(include_str!("settings.css")).expect("Unwrapping CSS should work!")}>
            <div class="setting">
                <label for="max-lines-per-slide">{"Max lines"}</label>
                <input
                    type="number"
                    id="max-lines-per-slide"
                    value={props.settings.max_lines_per_slide.to_string()}
                    oninput={Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        if let Ok(value) = input.value().parse::<u8>() {
                            if value >= 1 && value <= 10 {
                                set_max_lines_per_slide.emit(value);
                            }
                        }
                    })}
                />
            </div>
            <div class="setting">
                <label for="background">{"Background"}</label>
                <select id="background" onchange={Callback::from(move |e: Event| {
                    let select: HtmlSelectElement = e.target_unchecked_into();
                    if let Ok(value) = select.value().parse::<u8>() {
                        set_background.emit(value);
                    }
                })}>
                    <option value="0" selected={props.settings.background == 0}>{"Black"}</option>
                    <option value="1" selected={props.settings.background == 1}>{"Red"}</option>
                    <option value="2" selected={props.settings.background == 2}>{"Ray"}</option>
                </select>
            </div>
            <div class="setting">
                <label for="text-orientation">{"Text orientation"}</label>
                <select id="text-orientation" onchange={Callback::from(move |e: Event| {
                    let select: HtmlSelectElement = e.target_unchecked_into();
                    if let Ok(value) = select.value().parse::<SlideTextOrientation>() {
                        set_text_orientation.emit(value);
                    }
                })}>
                    <option value="top" selected={props.settings.text_orientation.to_select_value() == "top"}>{"Top"}</option>
                    <option value="center" selected={props.settings.text_orientation.to_select_value() == "center"}>{"Center"}</option>
                    <option value="bottom" selected={props.settings.text_orientation.to_select_value() == "bottom"}>{"Bottom"}</option>
                </select>
            </div>
            <div class="setting">
                <label for="font-size">{"Font size"}</label>
                <input type="number" id="font-size" value={props.settings.font_size.to_string()} oninput={Callback::from(move |e: InputEvent| {
                    let input: HtmlInputElement = e.target_unchecked_into();
                    if let Ok(value) = input.value().parse::<u8>() {
                        set_font_size.emit(value);
                    }
                })}/>
            </div>
            <div class="setting">
                <label for="horizontal-container-alignment">{"Horizontal container alignment"}</label>
                <select id="horizontal-container-alignment" onchange={Callback::from(move |e: Event| {
                    let select: HtmlSelectElement = e.target_unchecked_into();
                    if let Ok(value) = select.value().parse::<HorizontalContainerAlignment>() {
                        set_horizontal_container_alignment.emit(value);
                    }
                })}>
                    <option value="left" selected={props.settings.horizontal_container_alignment.to_select_value() == "left"}>{"Left"}</option>
                    <option value="center" selected={props.settings.horizontal_container_alignment.to_select_value() == "center"}>{"Center"}</option>
                    <option value="right" selected={props.settings.horizontal_container_alignment.to_select_value() == "right"}>{"Right"}</option>
                </select>
            </div>
            <div class="setting">
                <label for="text-alignment">{"Text alignment"}</label>
                <select id="text-alignment" onchange={Callback::from(move |e: Event| {
                    let select: HtmlSelectElement = e.target_unchecked_into();
                    if let Ok(value) = select.value().parse::<TextAlignment>() {
                        set_text_alignment.emit(value);
                    }
                })}>
                    <option value="left" selected={props.settings.text_alignment.to_select_value() == "left"}>{"Left"}</option>
                    <option value="center" selected={props.settings.text_alignment.to_select_value() == "center"}>{"Center"}</option>
                    <option value="right" selected={props.settings.text_alignment.to_select_value() == "right"}>{"Right"}</option>
                </select>
            </div>
        </div>
    }
}
