use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/components/editor/code_mirror_wrapper.js")]
extern "C" {
    pub type CodeMirrorWrapper;
    #[wasm_bindgen(constructor)]
    pub fn new() -> CodeMirrorWrapper;
    #[wasm_bindgen(method)]
    pub fn draw(
        this: &CodeMirrorWrapper,
        id: &str,
        onsave: Option<&Closure<dyn Fn(String)>>,
        onautoformat: Option<&Closure<dyn Fn(String) -> String>>,
    ) -> CodeMirrorWrapper;
    #[wasm_bindgen(method)]
    pub fn set_onsave(
        this: &CodeMirrorWrapper,
        onsave: Option<&Closure<dyn Fn(String)>>,
    ) -> CodeMirrorWrapper;
    #[wasm_bindgen(method)]
    pub fn set_onautoformat(
        this: &CodeMirrorWrapper,
        onautoformat: Option<&Closure<dyn Fn(String) -> String>>,
    ) -> CodeMirrorWrapper;
    #[wasm_bindgen(method)]
    pub fn set_content(this: &CodeMirrorWrapper, content: &str) -> CodeMirrorWrapper;
    #[wasm_bindgen(method)]
    fn get_content_bytes(this: &CodeMirrorWrapper) -> Vec<u8>;
    #[wasm_bindgen(method)]
    pub fn clean(this: &CodeMirrorWrapper);
    #[wasm_bindgen(method)]
    pub fn define_mode(
        this: &CodeMirrorWrapper,
        mode: &str,
        transitions_string: &str,
    ) -> CodeMirrorWrapper;
}

impl CodeMirrorWrapper {
    pub fn get_content(&self) -> String {
        String::from_utf8(self.get_content_bytes()).unwrap()
    }
}
