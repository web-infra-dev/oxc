import { basicSetup } from "codemirror";
import { EditorView, keymap, showPanel } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";

import init, { Oxc, OxcOptions } from "@oxc/wasm-web";

const placeholderText = `
import React, { useEffect, useRef } from 'react'

const DummyComponent:React.FC = () => {

  const ref = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (ref.current) ref.current.focus()
  }, [])

  return (
      <div>{Boolean(ref.current) ?? (
        <input type="text" ref={ref} />
      )}
      </div>
  )

}

export default DummyComponent
`.trim();

async function main() {
  await init();
  const oxc = new Oxc();
  const options = new OxcOptions();
  oxc.setOptions(options);
  oxc.setSourceText(placeholderText);
  oxc.run();
  const editor = initEditor(oxc);
  window.setTimeout(function () {
    editor.focus();
  });
}

main();

function initEditor(oxc) {
  function getAst() {
    return JSON.stringify(oxc.getAst(), null, 2);
  }

  function getHir() {
    return JSON.stringify(oxc.getHir(), null, 2);
  }

  function getFormattedText() {
    return oxc.getFormattedText()
  }

  function getMinifiedText() {
    return oxc.getMinifiedText()
  }

  function getConsole(_doc) {
    return oxc
      .getDiagnostics()
      .map((d) => d.message)
      .join("\n");
  }

  function consolePanel(view) {
    const dom = document.createElement("div");
    dom.textContent = getConsole(view.state.doc);
    return {
      dom,
      update(update) {
        if (update.docChanged) {
          dom.textContent = getConsole(update.state.doc);
          dom.scrollTop = dom.scrollHeight;
        }
      },
    };
  }

  const oxcLinter = linter((_view) => {
    return oxc.getDiagnostics().map((d) => ({
      from: d.start,
      to: d.end,
      severity: d.severity.toLowerCase(),
      message: d.message,
    }));
  });

  const rightView = new EditorView({
    doc: getAst(),
    extensions: [javascript(), githubDark, EditorView.editable.of(false), EditorView.lineWrapping],
    parent: document.querySelector("#display"),
  });

  function updateRightView(text) {
      const transaction = rightView.state.update({
        changes: { from: 0, to: rightView.state.doc.length, insert: text },
      });
      rightView.dispatch(transaction);
  }

  const stateListener = EditorView.updateListener.of((view) => {
    if (view.docChanged) {
      const sourceText = view.state.doc.toString();
      oxc.setSourceText(sourceText);
      oxc.run();
      updateRightView(getAst());
    }
  });

  document.querySelector("#ast").onclick = () => {
    updateRightView(getAst());
  };

  document.querySelector("#hir").onclick = () => {
    updateRightView(getHir());
  };

  document.querySelector("#format").onclick = () => {
    updateRightView(getFormattedText());
  };

  document.querySelector("#minify").onclick = () => {
    updateRightView(getMinifiedText());
  };

  const state = EditorState.create({
    doc: oxc.getSourceText(),
    extensions: [
      basicSetup,
      keymap.of(vscodeKeymap),
      javascript(),
      githubDark,
      lintGutter(),
      showPanel.of(consolePanel),
      oxcLinter,
      stateListener,
    ],
  });

  return new EditorView({
    state,
    parent: document.querySelector("#app"),
  });
}
