
export class CodeMirrorWrapper {
    clean() {
        this.editor.toTextArea();
        this.editor = undefined;
    }

    draw(id, onsave, onautoformat) {
        let extraKeys = {}
        if (onsave) {
            extraKeys["Ctrl-S"] = cm => onsave(cm.getValue());
            extraKeys["Cmd-S"] = cm => onsave(cm.getValue());
        }
        if (onautoformat) {
            extraKeys["Shift-Alt-F"] = cm => cm.setValue(onautoformat(cm.getValue()));
        }

        this.editor = new CodeMirror.fromTextArea(document.getElementById(id), {
            lineNumbers: true,
            extraKeys: extraKeys,
            mode: "generated",
            lineWrapping: true,
        });
        return this;
    }

    set_onsave(onsave) {
        if (this.editor) {
            this.editor.setOption("extraKeys", {
                ...this.editor.getOption("extraKeys"),
                "Ctrl-S": cm => onsave(cm.getValue()),
                "Cmd-S": cm => onsave(cm.getValue()),
            });
        }
        return this;
    }

    set_onautoformat(onautoformat) {
        if (this.editor) {
            this.editor.setOption("extraKeys", {
                ...this.editor.getOption("extraKeys"),
                "Shift-Alt-F": cm => cm.setValue(onautoformat(cm.getValue())),
            });
        }
        return this;
    }

    set_content(content) {
        this.editor.setValue(content);
        return this;
    }

    get_content_bytes() {
        const encoder = new TextEncoder();
        if (this.editor) {
            return encoder.encode(this.editor.getValue());
        } else {
            return encoder.encode("");
        }
    }

    define_mode(mode, transitions_string) {
        if (transitions_string === "") {
            return this;
        }

        const transitions = JSON.parse(transitions_string);
        CodeMirror.defineMode(mode, () => {
            return {
                startState: () => {
                    return {
                        state: "default",
                        parsed: "",
                    };
                },
                token: (stream, state) => {
                    while (true) {
                        const ch = stream.next();
                        if (ch === undefined) return null;

                        state.parsed += ch;

                        for (const transition of transitions) {
                            if (state.state === transition.state && state.parsed.endsWith(transition.suffix)) {
                                if (transition.back > 0) {
                                    state.parsed = state.parsed.slice(0, -transition.back);
                                    stream.backUp(transition.back);
                                }
                                if (transition.new_state !== null) state.state = transition.new_state;
                                if (transition.label !== null) {
                                    state.parsed = "";
                                    return transition.label;
                                }
                                break;
                            }
                        }
                    }
                },
            };
        });
        return this;
    }
}
