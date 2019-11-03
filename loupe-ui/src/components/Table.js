import React, { useState, useEffect } from "react";
import Axios from "axios";

export default function Table({ query }) {
  const [records, setRecords] = useState([]);

  useEffect(() => {
    const source = Axios.CancelToken.source();

    Axios.post(
      "http://localhost:9292/",
      { query },
      { cancelToken: source.token }
    )
      .then(response => setRecords(response.data))
      .catch(error => {
        if (!Axios.isCancel(error)) {
          throw error;
        }
      });

    return () => source.cancel();
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
