import React, { useEffect, useState } from "react";
import "./App.css";
import { Routes, Route } from "react-router-dom";
import Home from "./components/Home";
import Plugin from "./components/Plugin";
import IConnection from "./types/Connection";
import ConnectionService from "./services/ConnectionService";
import "bootstrap/dist/css/bootstrap.min.css";

const App: React.FC = () => {
  const [connections, setConnections] = useState<Array<IConnection>>([]);

  useEffect(() => {
    retrieveConnections();
  }, []);

  const retrieveConnections = () => {
    ConnectionService.getAll()
      .then((response: any) => {
        setConnections(response.data);
      })
      .catch((e: Error) => {
        console.log(e);
      });
  };

  return (
    <div className="App">
      <header className="App-header">
        <p>Query Plugins</p>
      </header>
      <div className="m-1">
        <Routes>
          <Route path="/" element={<Home connections={connections} />} />
          <Route
            path="/plugins/:name"
            element={<Plugin connections={connections} />}
          />
        </Routes>
      </div>
    </div>
  );
};

export default App;
