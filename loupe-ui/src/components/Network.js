import React, { useState, useEffect } from "react";
import Axios from "axios";

export default function Network({ query }) {
  const [nodes, setNodes] = useState({});

  useEffect(() => {
    Axios.get(`http://localhost:9191/debug/nodes`, {
      params: { query }
    }).then(response => setNodes(response.data));
  }, [query]);

  return <p>{JSON.stringify(nodes)}</p>;
}
