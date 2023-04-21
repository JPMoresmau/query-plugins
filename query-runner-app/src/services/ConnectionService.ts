import http from "../http-common";
import IConnection from "../types/Connection";

const getAll = () => {
  return http.get<Array<IConnection>>("/connections");
};

const ConnectionService = {
    getAll,
  };
  
export default ConnectionService;
