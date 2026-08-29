import { createRoot } from "react-dom/client";
import { StrictMode } from "react";
import App from "../App";
import { installMockBackend } from "./mock-backend";
import "../index.css";

installMockBackend();

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
