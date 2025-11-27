use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, Window};

fn get_notifications_div(document: &Document) -> Result<Element, JsValue> {
    if let Some(existing) = document.query_selector(".fancy-yew-toast-notifications")? {
        return Ok(existing);
    }

    let notifications_div = document.create_element("div")?;
    notifications_div.set_class_name("fancy-yew-toast-notifications");
    document.body().unwrap().append_child(&notifications_div)?;

    Ok(notifications_div)
}

#[wasm_bindgen]
pub fn create_toast(toast_type: &str, icon: &str, title: &str, text: &str) {
    let window: Window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");

    let new_toast = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    let html = format!(
        r#"<div class="fancy-yew-toast-notification {toast_type}">
                <span class="material-symbols-outlined icon">{icon}</span>
                <div class="content">
                    <div class="title">{title}</div>
                    <span>{text}</span>
                </div>
                <span class="material-symbols-outlined icon close-btn">close</span>
           </div>"#,
        toast_type = toast_type,
        icon = icon,
        title = title,
        text = text
    );

    new_toast.set_inner_html(&html);

    if let Some(close_btn) = new_toast
        .query_selector(".close-btn")
        .unwrap()
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        let toast_clone = new_toast.clone();
        let closure = Closure::wrap(Box::new(move || {
            toast_clone.remove();
        }) as Box<dyn Fn()>);
        close_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    if let Ok(notifications) = get_notifications_div(&document) {
        notifications.append_child(&new_toast).unwrap();
    }

    let toast_clone = new_toast.clone();
    let closure = Closure::wrap(Box::new(move || {
        toast_clone.remove();
    }) as Box<dyn Fn()>);

    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            5000,
        )
        .unwrap();
    closure.forget();
}

#[allow(dead_code)]
pub fn show_success(title: &str, message: &str) {
    create_toast("success", "check_circle", title, message);
}

#[allow(dead_code)]
pub fn show_error(title: &str, message: &str) {
    create_toast("error", "bug_report", title, message);
}

#[allow(dead_code)]
pub fn show_warning(title: &str, message: &str) {
    create_toast("warning", "warning", title, message);
}

#[allow(dead_code)]
pub fn show_info(title: &str, message: &str) {
    create_toast("info", "info", title, message);
}
