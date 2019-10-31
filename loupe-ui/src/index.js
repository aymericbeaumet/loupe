import App from "./components/App";
import Container from "@material-ui/core/Container";
import CssBaseline from "@material-ui/core/CssBaseline";
import React from "react";
import ReactDOM from "react-dom";

ReactDOM.render(
  <>
    <CssBaseline />
    <Container maxWidth="sm">
      <App />
    </Container>
  </>,
  document.getElementById("app")
);
