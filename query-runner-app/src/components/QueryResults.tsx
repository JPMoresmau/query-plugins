import { IQueryResult } from "../types/Plugin";

interface ResultsProps {
  results: IQueryResult;
}

function QueryResults(props: ResultsProps) {
  return (
    <table>
      <thead>
        <tr>
          {props.results.names.map((n, ix) => (
            <th key={"col" + ix}>{n}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {props.results.values.map((row, ix1) => (
          <tr key={"row" + ix1}>
            {row.map((v, ix2) => (
              <td key={"val" + ix1 + "-" + ix2}>{v}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export default QueryResults;
