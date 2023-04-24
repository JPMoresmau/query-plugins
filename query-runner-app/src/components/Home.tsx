import React from "react";

import ConnectionsList from "./ConnectionList";
import PluginsList from "./PluginList";
import IConnection from "../types/Connection";

interface HomeProps {
  connections: IConnection[];
}

function Home(props: HomeProps) {
  return (
    <div className="row">
      <div className="col">
        <p>Connections</p>
        <ConnectionsList connections={props.connections} />
      </div>
      <div className="col">
        <p>Plugins</p>
        <PluginsList />
      </div>
    </div>
  );
}

export default Home;
