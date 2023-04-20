# Data plugins in WebAssembly

This project is a POC of using WebAssembly to develop plugins that can interact with backend databases by running queries and processing results.

Plugins are responsible for defining their input parameters, and define the SQL to run. They can then process each row of data and do
whatever operations they need.

The runtime is responsible for actually connecting to the database, running the SQL and passing the data returned from the database to the plugins.

## Plugin interface

We use [WAI](https://github.com/wasmerio/wai) to define the interface for the plugins. See (query-common.wai)[query-runner/query-common.wai] 
for basic types and (query.wai)[query-runner/query.wai] for the main interface for the plugins. Really they need to implement:
- the `metadata` method to return a description of what the query does and its input parameters.
- the `start` method to start an execution of a query, given actual values for the input parameters.
- the `execution` resource that contains the actual SQL to run, the bound parameters for the SQL, and methods to handle each row
of data (`row`) and the `end` of the query.

(test-collect)[test-collect] is a test plugin that runs a simple query and capture the rows without doing any processing, and can give you
an idea on how to use the Wasmer `export!` macro to generate the traits that need to be implemented by your code.

### Design choices

We currently support four datatypes (boolean, integer, decimal and string) and there's a `timestamp` data type that for the moment is just a string.

The SQL query and actual bound parameters for it can be generated dynamically in the `start` method, so the plugin can generate SQL dynamically when
need be and still have bound parameters.

The `end` function is always called even if no data was returned, and it's passed the name of columns so these names can be returned even if no data was
returned.

## Runtime

The runtime is a library in [query-runner](query-runner) and a simple command line executable is provided in [query-runner-bin](query-runner-bin).
Currently it only loads connection information from a [file](query-runner/config/connections.yaml) and plugins from a [folder](query-runner/plugins).

Only sqlite and postgres (without TLS) are currently supported as a backing databases. This is a very early prototype!
