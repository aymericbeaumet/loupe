import React, { useEffect, useRef } from "react";
import Axios from "axios";
import cytoscape from "cytoscape";
import dagre from "cytoscape-dagre";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

cytoscape.use(dagre);

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
      .then(response => {
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
            classes: "node",
            data: {
              id: node.id,
              content: decoder.decode(Uint8Array.from(node.path))
            }
          });
          for (const [key, child] of node.children) {
            child.id = nextId++;
            child.path = [...node.path, key];
            elements.push({
              data: { source: node.id, target: child.id }
            });
            stack.push(child);
          }
          for (const record of node.records) {
            const recordId = `record:${record.id}`;
            elements.push({
              classes: "record",
              data: {
                id: recordId,
                content: record.name
              }
            });
            elements.push({
              data: {
                source: node.id,
                target: recordId
              }
            });
          }
        }
        cyRef.current = cytoscape({
          container: divRef.current,
          elements,
          autoungrabify: true,
          autounselectify: true,
          layout: {
            name: "dagre",
            nodeDimensionsIncludeLabels: true
          },
          style: [
            {
              selector: "node",
              style: {
                content: "data(content)",
                "text-valign": "center"
              }
            },
            {
              selector: "node.record",
              style: {
                "background-color": "green"
              }
            },
            {
              selector: "edge",
              style: {
                width: 1,
                "target-arrow-shape": "triangle"
              }
            }
          ]
        });
      })
      .catch(error => {
        if (!Axios.isCancel(error)) {
          throw error;
        }
      });

    return () => {
      source.cancel();
      if (cyRef.current) {
        cyRef.current.stop();
      }
    };
  }, [query]);

  return <div ref={divRef} style={{ height: "100vh" }}></div>;
}
