import React from "react";
import Axios from "axios";

class App extends React.Component {
  state = {
    records: []
  };

  handleChange = event => {
    const query = event.target.value;
    if (!query) {
      this.setState({ records: [] });
    } else {
      Axios.post("http://localhost:9292/", { query }).then(response =>
        this.setState({ records: response.data })
      );
    }
  };

  render() {
    return (
      <>
        <input type="text" autoFocus={true} onChange={this.handleChange} />
        <table>
          <tbody>
            {this.state.records.map(record => (
              <tr key={record.id}>
                <td>{JSON.stringify(record)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </>
    );
  }
}

export default App;
