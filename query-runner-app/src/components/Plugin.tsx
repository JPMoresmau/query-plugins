import { useState, useEffect, ChangeEvent } from "react";
import { useParams, Link } from "react-router-dom";
import { IPluginMetadata, IPluginRun, IQueryResult } from "../types/Plugin";
import PluginService from "../services/PluginService";
import IConnection from "../types/Connection";
import axios, { AxiosError } from "axios";
import QueryResults from "./QueryResults";

interface PluginProps {
  connections: IConnection[];
}

function Plugin(props: PluginProps) {
  const { name } = useParams();

  const [metadata, setMetadata] = useState<IPluginMetadata>();
  const [variables, setVariables] = useState<{ [k: string]: any }>({});
  const [connection, setConnection] = useState<string>();
  const [results, setResults] = useState<IQueryResult>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    if (name) {
      retrieveMetadata(name);
    }
  }, [name]);

  const retrieveMetadata = (name: string) => {
    PluginService.getMetadata(name)
      .then((response: any) => {
        setMetadata(response.data);
      })
      .catch((e: Error | AxiosError) => {
        console.log(e);
        if (axios.isAxiosError(e)) {
          setError(e.response?.data.error);
        } else {
          setError(e.toString());
        }
      });
  };

  const handleInputChange = (event: ChangeEvent<HTMLInputElement>) => {
    const { name, value } = event.target;
    setVariables({ ...variables, [name]: value });
  };
  const handleConnectionChange = (event: ChangeEvent<HTMLSelectElement>) => {
    setConnection(event.target.value);
  };
  const runPlugin = () => {
    setError("");
    let run: IPluginRun = {
      plugin: name || "",
      connection: connection || props.connections[0]?.name,
      variables,
    };
    console.log(run);
    PluginService.run(run)
      .then((response: any) => {
        setResults(response.data);
      })
      .catch((e: Error | AxiosError) => {
        console.log(e);
        if (axios.isAxiosError(e)) {
          setError(e.response?.data.error);
        } else {
          setError(e.toString());
        }
      });
    return false;
  };

  return (
    <div>
      <div>
        <Link to="/">Home</Link> / Plugin <b>{name}</b>
      </div>
      <div className="plugin-description p-2">
        {metadata?.description || ""}
      </div>

      <div className="row">
        <div className="col">
          <form>
            <div className="connection">
              <label htmlFor="connection">Run with connection:</label>
              <select
                onChange={handleConnectionChange}
                id="connection"
                name="connection"
                className="form-select"
              >
                {props.connections &&
                  props.connections.map((conn) => (
                    <option key={conn.name} value={conn.name}>
                      {conn.name}
                    </option>
                  ))}
              </select>
            </div>
            <div className="params">
              {metadata &&
                metadata.parameters &&
                metadata.parameters.map((param) => (
                  <div className="mb-3" key={param.name}>
                    <label htmlFor={param.name}>{param.name}:</label>
                    <input
                      type="text"
                      className="form-control"
                      id={param.name}
                      required
                      name={param.name}
                      onChange={handleInputChange}
                    />
                  </div>
                ))}
            </div>
            <button
              type="button"
              onClick={runPlugin}
              className="btn btn-primary"
            >
              Run
            </button>
          </form>
        </div>
        <div className="col">
          <div>Results</div>
          {results && <QueryResults results={results} />}
        </div>
      </div>
      {error && (
        <div
          className="alert alert-danger alert-dismissible fade show"
          role="alert"
        >
          {error}
          <button
            type="button"
            className="btn-close"
            data-bs-dismiss="alert"
            aria-label="Close"
          ></button>
        </div>
      )}
    </div>
  );
}

export default Plugin;
