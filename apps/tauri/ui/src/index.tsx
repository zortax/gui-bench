/* @refresh reload */
// The page of the Tauri app: the scenario that the command line of the app names.
import { render } from "solid-js/web";

import { App } from "./App";
import "./dashboard/dashboard.css";

render(() => <App />, document.getElementById("root")!);
