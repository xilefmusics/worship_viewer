use stylist::Style;
use yew::prelude::*;
use std::str::FromStr;
use super::SettingsData;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum SlideTextOrientation {
    Top,
    #[default]
    Center,
    Bottom,
}

impl SlideTextOrientation {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Top => "text-orientation-top",
            Self::Center => "text-orientation-center",
            Self::Bottom => "text-orientation-bottom",
        }
    }

    pub fn to_select_value(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Center => "center",
            Self::Bottom => "bottom",
        }
    }
}

impl FromStr for SlideTextOrientation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "top" => Ok(Self::Top),
            "center" => Ok(Self::Center),
            "bottom" => Ok(Self::Bottom),
            _ => Err(()),
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum HorizontalContainerAlignment {
    Left,
    #[default]
    Center,
    Right,
}

impl HorizontalContainerAlignment {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Left => "container-align-left",
            Self::Center => "container-align-center",
            Self::Right => "container-align-right",
        }
    }

    pub fn to_select_value(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

impl FromStr for HorizontalContainerAlignment {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum TextAlignment {
    Left,
    #[default]
    Center,
    Right,
}

impl TextAlignment {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Left => "text-align-left",
            Self::Center => "text-align-center",
            Self::Right => "text-align-right",
        }
    }

    pub fn to_select_value(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

impl FromStr for TextAlignment {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

#[derive(Properties, Serialize, Deserialize, Clone, PartialEq)]
pub struct SlideProps {
    #[prop_or_default]
    pub text: String,
    #[prop_or_default]
    pub settings: SettingsData,
    #[prop_or_default]
    pub is_black: bool,
    #[prop_or_default]
    pub expand: bool,
}

#[function_component(Slide)]
pub fn slide(props: &SlideProps) -> Html {
    html! {
        <div class={classes!{
            Style::new(include_str!("slide.css")).expect("Unwrapping CSS should work!"),
            format!("background-{}", if props.is_black { 0 } else { props.settings.background }),
            props.settings.text_orientation.to_str(),
            props.settings.horizontal_container_alignment.to_str(),
            if props.expand { "expand" } else { "" },
        }}>
            <div 
                class={classes!("text-container", props.settings.text_alignment.to_str())}
                style={format!("padding: {}cqw", props.settings.font_size as f32 / 19.2 * 2.0)}
            >
                { for props.text.lines().map(|line| html! { 
                    <div 
                        class="line"
                        style={format!("font-size: {}cqw", props.settings.font_size as f32 / 19.2)}
                    >
                        {line}
                    </div>
                }) }
            </div>
        </div>
    }
}
