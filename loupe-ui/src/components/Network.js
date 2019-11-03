import React, { useEffect, useRef } from "react";
import Axios from "axios";
import cytoscape from "cytoscape";
import dagre from "cytoscape-dagre";

cytoscape.use(dagre);

export default function Network({ query }) {
  const divRef = useRef();

  useEffect(() => {
    const source = Axios.CancelToken.source();

    Axios.get(`http://localhost:9191/debug/nodes`, {
      params: { query },
      cancelToken: source.token
    })
      .then(response => {
        let nextId = 0;
        const stack = response.data
          ? [
              {
                id: nextId++,
                path: [],
                ...response.data
              }
            ]
          : [];
        const elements = [];
        let node;
        while ((node = stack.pop())) {
          elements.push({
            data: {
              id: node.id,
              label: JSON.stringify(node.path)
            }
          });
          for (const [key, child] of node.children) {
            child.id = nextId++;
            child.path = [...node.path, key];
            elements.push({
              data: {
                source: node.id,
                target: child.id
              }
            });
            stack.push(child);
          }
        }
        cytoscape({
          container: divRef.current,
          elements,
          layout: {
            name: "dagre",
            nodeDimensionsIncludeLabels: true
          },
          style: [
            {
              selector: "node",
              style: {
                label: "data(label)"
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

    return () => source.cancel();
  }, [query]);

  return <div ref={divRef} style={{ height: "100vh" }}></div>;
}
