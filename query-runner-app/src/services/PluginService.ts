import http from "../http-common";
import IPlugin, { IPluginMetadata, IPluginRun, IQueryResult } from "../types/Plugin";

const getAll = () => {
  return http.get<Array<IPlugin>>("/plugins");
};

const getMetadata = (name: string) => {
  return http.get<IPluginMetadata>("/plugins/" + encodeURIComponent(name));
};

const run = (run: IPluginRun) => {
  return http.post<IQueryResult>("/plugins/" + encodeURIComponent(run.plugin)+"/"+encodeURIComponent(run.connection), run.variables);
}

const PluginService = {
    getAll,
    getMetadata,
    run
  };
  
export default PluginService;
