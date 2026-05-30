import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles/colors_and_type.css";
import "./styles/app.css";
import "./styles/root.css";

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
