use super::NavItemBuilder;
use yew::Html;

pub trait Navable
where
    Self: Sized,
{
    fn route_items() -> Vec<Self>;
    fn to_nav_item(self) -> NavItemBuilder<'static>;
    fn render(route: Self) -> Html;
}
