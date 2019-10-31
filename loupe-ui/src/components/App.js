import React from "react";
import Axios from "axios";
import TextField from "@material-ui/core/TextField";

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
      <form autoComplete="off">
        <TextField
          id="outlined-basic"
          label="Search a record"
          margin="normal"
          variant="outlined"
          autoFocus={true}
          onChange={this.handleChange}
        />
        <table>
          <tbody>
            {this.state.records.map(record => (
              <tr key={record.id}>
                <td>{JSON.stringify(record)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </form>
    );
  }
}

export default App;
