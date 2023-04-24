import React, { useState, useEffect } from "react";
import { Link } from "react-router-dom";

import PluginService from "../services/PluginService";
import IPlugin from "../types/Plugin";

const PluginsList: React.FC = () => {
  const [plugins, setPlugins] = useState<Array<IPlugin>>([]);

  useEffect(() => {
    retrievePlugins();
  }, []);

  const retrievePlugins = () => {
    PluginService.getAll()
      .then((response: any) => {
        setPlugins(response.data);
      })
      .catch((e: Error) => {
        console.log(e);
      });
  };

  return (
    <ul>
      {plugins &&
        plugins.map((plugin) => (
          <li key={plugin.name}>
            <Link to={"/plugins/" + plugin.name}>{plugin.name}</Link>:{" "}
            <span className="plugin-description">{plugin.description}</span>
          </li>
        ))}
    </ul>
  );
};

export default PluginsList;
