import React, { useRef } from "react";
import Table from "./Table";
import Network from "./Network";
import { useQueryParam, StringParam, BooleanParam } from "use-query-params";

export default function App() {
  const inputTextRef = useRef(null);
  const [query = "", setQuery] = useQueryParam("query", StringParam);
  const [network = false, setNetwork] = useQueryParam("network", BooleanParam);

  let normalizedQuery = query.trim();

  return (
    <>
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
          setNetwork(event.target.checked);
          inputTextRef.current.focus();
        }}
        checked={network}
      />
      {network ? (
        <Network query={normalizedQuery} />
      ) : (
        <Table query={normalizedQuery} />
      )}
    </>
  );
}
