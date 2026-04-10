
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
                        /** Text inside the current `[` … `]` chord span (chordlib / Nashville / duration). */
                        chordAcc: "",
                        /** After `{meta:`, color tag name vs value (`key` → blue, `value` → green). */
                        metaSplitPhase: null,
                        /** True once a non-whitespace key character was emitted (leading spaces after `:` are skipped). */
                        metaKeySeenNonWs: false,
                        /** Line begins with optional spaces then `&` (chordlib translation line). */
                        translationLine: false,
                        /** The next `&` on this translation line is the marker (green), not lyric text. */
                        translationPendingAmp: false,
                        /** Chord span was opened from a `&…` translation line (darker chord yellow). */
                        chordOnTranslationLine: false,
                    };
                },
                token: (stream, state) => {
                    while (true) {
                        // {meta: tag value} — tag same green as directives (meta-tag-key), value gray like translation lines (meta-keypair-value)
                        if (state.state === "meta-value" && state.metaSplitPhase === "key") {
                            const ch = stream.next();
                            if (ch === undefined) return null;
                            if (ch === "}") {
                                stream.backUp(1);
                                state.metaSplitPhase = null;
                                state.metaKeySeenNonWs = false;
                                continue;
                            }
                            if (/\s/.test(ch)) {
                                if (!state.metaKeySeenNonWs) {
                                    state.parsed = "";
                                    return null;
                                }
                                state.metaSplitPhase = "value";
                                state.metaKeySeenNonWs = false;
                                state.parsed = "";
                                return null;
                            }
                            state.metaKeySeenNonWs = true;
                            state.parsed = "";
                            return "meta-tag-key";
                        }
                        if (state.state === "meta-value" && state.metaSplitPhase === "value") {
                            const ch = stream.next();
                            if (ch === undefined) return null;
                            if (ch === "}") {
                                stream.backUp(1);
                                state.metaSplitPhase = null;
                                state.metaKeySeenNonWs = false;
                                continue;
                            }
                            state.parsed = "";
                            return "meta-keypair-value";
                        }

                        // Lines like `&[E]Du hast…` — marker `&` green, lyrics gray; `[…]` chords use darker yellow.
                        if (state.state === "default") {
                            if (stream.sol()) {
                                const isTrans = stream.match(/^[ \t]*&/, false) != null;
                                state.translationLine = isTrans;
                                state.translationPendingAmp = isTrans;
                            }
                        }
                        if (state.state === "default" && state.translationLine) {
                            const p = stream.peek();
                            if (p === "\n") {
                                stream.next();
                                state.translationLine = false;
                                state.translationPendingAmp = false;
                                state.parsed = "";
                                return null;
                            }
                            if (p !== "[" && p !== "{") {
                                const chT = stream.next();
                                state.parsed = "";
                                if (chT === "&" && state.translationPendingAmp) {
                                    state.translationPendingAmp = false;
                                    state.translationLine = true;
                                    return "meta-value";
                                }
                                return "translation-lyric";
                            }
                        }

                        const ch = stream.next();
                        if (ch === undefined) return null;

                        const prevState = state.state;
                        state.parsed += ch;

                        if (state.state === "chord" && ch !== "]") {
                            state.chordAcc += ch;
                        }

                        for (const transition of transitions) {
                            if (state.state === transition.state && state.parsed.endsWith(transition.suffix)) {
                                if (transition.back > 0) {
                                    state.parsed = state.parsed.slice(0, -transition.back);
                                    stream.backUp(transition.back);
                                }
                                if (transition.new_state !== null) {
                                    state.state = transition.new_state;
                                    if (transition.state === "meta-key" && transition.new_state === "meta-middle") {
                                        state.metaSplitPhase =
                                            transition.suffix === "meta:" ? "key" : null;
                                        if (state.metaSplitPhase === "key") {
                                            state.metaKeySeenNonWs = false;
                                        }
                                    }
                                    if (transition.new_state === "meta-end") {
                                        state.metaSplitPhase = null;
                                        state.metaKeySeenNonWs = false;
                                    }
                                    if (transition.new_state === "default") {
                                        state.metaSplitPhase = null;
                                        state.metaKeySeenNonWs = false;
                                    }
                                    if (transition.new_state === "chord" && prevState !== "chord") {
                                        state.chordAcc = "";
                                    }
                                    if (transition.state === "default" && transition.new_state === "meta-begin") {
                                        state.metaSplitPhase = null;
                                        state.metaKeySeenNonWs = false;
                                    }
                                    if (transition.state === "default" && transition.new_state === "chord") {
                                        state.chordOnTranslationLine = state.translationLine;
                                    }
                                }
                                if (transition.label !== null) {
                                    if (
                                        state.metaSplitPhase === "key" &&
                                        transition.state === "meta-middle" &&
                                        transition.new_state === "meta-value" &&
                                        transition.suffix === ""
                                    ) {
                                        state.parsed = "";
                                        break;
                                    }
                                    let label = transition.label;
                                    if (
                                        label === "chord" &&
                                        transition.suffix === "]" &&
                                        state.chordAcc.length > 0 &&
                                        /^\d/.test(state.chordAcc)
                                    ) {
                                        label = "nashville";
                                    }
                                    if (state.chordOnTranslationLine) {
                                        if (
                                            label === "default" &&
                                            transition.state === "default" &&
                                            transition.new_state === "chord" &&
                                            transition.suffix === "["
                                        ) {
                                            label = "chord-translation";
                                        } else if (label === "chord") {
                                            label = "chord-translation";
                                        } else if (label === "nashville") {
                                            label = "nashville-translation";
                                        }
                                    }
                                    if (transition.suffix === "]") {
                                        state.chordAcc = "";
                                    }
                                    if (transition.state === "chord" && transition.new_state === "default") {
                                        state.chordOnTranslationLine = false;
                                    }
                                    state.parsed = "";
                                    return label;
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
