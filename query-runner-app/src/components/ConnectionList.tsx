import IConnection from "../types/Connection";

interface ConnectionProps {
  connections: IConnection[];
}

function ConnectionsList(props: ConnectionProps) {
  return (
    <ul>
      {props.connections &&
        props.connections.map((conn) => (
          <li key={conn.name}>
            {conn.name} <i>({conn.db_type})</i>
          </li>
        ))}
    </ul>
  );
}

export default ConnectionsList;
