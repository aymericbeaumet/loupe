import React, { useState, useRef } from "react";
import Table from "./Table";
import Network from "./Network";

export default function App() {
  const inputTextRef = useRef(null);
  const [showNetwork, setShowNetwork] = useState(false);
  const [query, setQuery] = useState("");

  return (
    <>
      <form autoComplete="off">
        <input
          ref={inputTextRef}
          type="text"
          onChange={event => setQuery(event.target.value)}
          value={query}
          placeholder="search records..."
          autoFocus={true}
        />
        <input
          type="checkbox"
          onChange={event => {
            setShowNetwork(event.target.checked);
            inputTextRef.current.focus();
          }}
          value={showNetwork}
        />
      </form>
      {showNetwork ? <Network query={query} /> : <Table query={query} />}
    </>
  );
}
