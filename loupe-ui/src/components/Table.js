import React, { useState, useEffect } from "react";
import Axios from "axios";

export default function Table({ query }) {
  const [records, setRecords] = useState([]);

  useEffect(() => {
    Axios.post("http://localhost:9292/", {
      query
    }).then(response => setRecords(response.data));
  }, [query]);

  return (
    <table>
      <tbody>
        {records.map(record => (
          <tr key={record.id}>
            <td>{JSON.stringify(record)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
