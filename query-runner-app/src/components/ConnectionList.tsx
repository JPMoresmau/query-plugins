import React, { useState, useEffect } from "react";

import ConnectionService from "../services/ConnectionService";
import IConnection from "../types/Connection";

const ConnectionsList: React.FC = () => {
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
    <ul>
      {connections &&
        connections.map((conn) => (
          <li key={conn.name}>
            {conn.name} <i>({conn.db_type})</i>
          </li>
        ))}
    </ul>
  );
};

export default ConnectionsList;
