export default interface IPlugin {
  name: string;
  description: string;
}

export interface IParameter {
  name: string;
  parameter_type: string;
}

export interface IPluginMetadata {
  name: string;
  description: string;
  parameters: IParameter[];
}

export interface IQueryResult {
  names: string[];
  values: any[][];
}

export interface IPluginRun {
  plugin: string;
  connection: string;
  variables: { [k: string]: any };
}
