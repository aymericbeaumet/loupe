import React, { useState } from "react";
import Table from "./Table";
import Network from "./Network";

export default function App() {
  const [showNetwork, setShowNetwork] = useState(false);
  const [query, setQuery] = useState("");

  return (
    <>
      <form autoComplete="off">
        <input
          type="text"
          onChange={event => setQuery(event.target.value)}
          value={query}
          placeholder="search records..."
          autoFocus={true}
        />
        <input
          type="checkbox"
          onChange={event => setShowNetwork(event.target.checked)}
          value={showNetwork}
        />
      </form>
      {showNetwork ? <Network query={query} /> : <Table query={query} />}
    </>
  );
}
