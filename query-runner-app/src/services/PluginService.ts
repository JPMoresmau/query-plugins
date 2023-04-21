import http from "../http-common";
import IPlugin from "../types/Plugin";

const getAll = () => {
  return http.get<Array<IPlugin>>("/plugins");
};

const PluginService = {
    getAll,
  };
  
export default PluginService;
