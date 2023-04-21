import React from "react";
import "./App.css";
import ConnectionsList from "./components/ConnectionList";
import PluginsList from "./components/PluginList";

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <p>Query Plugins</p>
      </header>
      <div className="row">
        <div className="column">
          <p>Connections</p>
          <ConnectionsList />
        </div>
        <div className="column">
          <p>Plugins</p>
          <PluginsList/>
        </div>
      </div>
    </div>
  );
}

export default App;
