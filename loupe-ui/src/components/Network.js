import React, { useEffect, useRef } from "react";
import Axios from "axios";
import cytoscape from "cytoscape";
import dagre from "cytoscape-dagre";
import popper from "cytoscape-popper";
import tippy from "tippy.js";

cytoscape.use(dagre);
cytoscape.use(popper);

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export default function Network({ query }) {
  const cyRef = useRef(null);
  const divRef = useRef(null);

  useEffect(() => {
    const source = Axios.CancelToken.source();

    divRef.current.innerHTML = "";

    Axios.get("http://localhost:9191/debug/nodes", {
      params: { query },
      cancelToken: source.token
    })
      .then(buildElements)
      .then(elements => {
        cyRef.current = renderNetwork({
          container: divRef.current,
          elements
        });
      })
      .catch(error => {
        if (!Axios.isCancel(error)) {
          throw error;
        }
      });

    return () => {
      source.cancel();
      cyRef.current && cyRef.current.destroy();
    };
  }, [query]);

  return <div ref={divRef} style={{ height: "100vh" }}></div>;
}

function buildElements(response) {
  let nextId = 0;
  const stack = Object.entries(response.data).map(([word, node]) => ({
    id: nextId++,
    path: encoder.encode(word),
    ...node
  }));
  const elements = [];
  let node;
  while ((node = stack.pop())) {
    elements.push({
      classes: "byte",
      data: {
        id: node.id,
        byte: {
          path: node.path,
          content: utf8ToUtf16(node.path)
        }
      }
    });
    for (const [key, child] of node.children) {
      child.id = nextId++;
      child.path = [...node.path, key];
      elements.push({ data: { source: node.id, target: child.id } }); // edge
      stack.push(child);
    }
    for (const record of node.records) {
      const recordId = `r:${record.id}`;
      elements.push({
        classes: "record",
        data: {
          id: recordId,
          record
        }
      });
      elements.push({ data: { source: node.id, target: recordId } }); // edge
    }
  }
  return elements;
}

function renderNetwork({ container, elements }) {
  const cy = cytoscape({
    container,
    elements,
    autoungrabify: true,
    motionBlur: true,
    layout: {
      name: "dagre",
      nodeDimensionsIncludeLabels: true
    },
    style: [
      {
        selector: "node",
        style: {
          "text-valign": "center",
          "background-color": "white",
          "border-width": 1,
          "border-color": "black"
        }
      },
      {
        selector: "node.byte",
        style: {
          content: "data(byte.content)"
        }
      },
      {
        selector: "node.record",
        style: {
          content: "data(record.id)",
          "background-color": "green"
        }
      },
      {
        selector: "edge",
        style: {
          width: 1,
          "curve-style": "bezier",
          "target-arrow-shape": "triangle"
        }
      },
      {
        selector: ".highlight",
        style: {
          // node
          "border-width": 2,
          "border-color": "orange",
          // edge
          "line-color": "orange",
          "target-arrow-shape": "triangle",
          "target-arrow-color": "orange"
        }
      }
    ]
  });

  cy.ready(() => {
    cy.elements("node")
      .unbind("select")
      .bind("select", ({ target: node }) => {
        node.addClass("highlight");
        node.predecessors().forEach(p => p.addClass("highlight"));
        node.successors().forEach(s => s.addClass("highlight"));
      })
      .unbind("unselect")
      .bind("unselect", ({ target: node }) => {
        node.removeClass("highlight");
        node.predecessors().forEach(p => p.removeClass("highlight"));
        node.successors().forEach(s => s.removeClass("highlight"));
      });

    cy.elements("edge")
      .unbind("select")
      .bind("select", ({ target: node }) => {
        node.addClass("highlight");
        node.connectedNodes().forEach(n => n.addClass("highlight"));
      })
      .unbind("unselect")
      .bind("unselect", ({ target: node }) => {
        node.removeClass("highlight");
        node.connectedNodes().forEach(n => n.removeClass("highlight"));
      });

    cy.elements("node.byte")
      .unbind("mouseover")
      .bind("mouseover", ({ target: byte }) => {
        if (!byte.tippy) {
          byte.tippy = tippy(byte.popperRef(), {
            content() {
              const div = document.createElement("div");
              div.innerHTML = `[${byte
                .data("byte")
                .path.map(n => `0x${n.toString(16).toUpperCase()}`)
                .join(", ")}]`;
              return div;
            },
            placement: "top",
            trigger: "manual",
            hideOnClick: false,
            multiple: true,
            sticky: true
          });
        }
        byte.tippy.show();
      })
      .unbind("mouseout remove")
      .bind("mouseout", ({ target: byte }) => byte.tippy.hide());

    cy.elements("node.record")
      .unbind("mouseover")
      .bind("mouseover", ({ target: record }) => {
        if (!record.tippy) {
          record.tippy = tippy(record.popperRef(), {
            content() {
              const div = document.createElement("div");
              div.innerHTML = JSON.stringify(record.data("record"));
              return div;
            },
            placement: "bottom",
            trigger: "manual",
            hideOnClick: false,
            multiple: true,
            sticky: true
          });
        }
        record.tippy.show();
      })
      .unbind("mouseout remove")
      .bind("mouseout", ({ target: record }) => record.tippy.hide());
  });

  cy.on("destroy", () => {
    cy.elements().forEach(element => {
      if (element.tippy) {
        element.tippy.destroy();
      }
    });
  });

  return cy;
}

// Properly replace � with its hexadecimal counterpart
function utf8ToUtf16(utf8) {
  const utf16 = decoder.decode(Uint8Array.from(utf8));
  let count = 0;
  while (utf16[utf16.length - count - 1] === "\ufffd") {
    count++;
  }
  if (count > 0) {
    return `${utf16.slice(0, utf16.length - count)}[${utf8
      .slice(-count)
      .map(i => `0x${i.toString(16).toUpperCase()}`)
      .join(", ")}]`;
  }
  return utf16;
}
