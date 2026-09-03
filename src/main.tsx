import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "./App";
import { MenuWindow } from "./components/menu-window";
import { TabMenuWindow } from "./components/tab-menu-window";
import "./index.css";

// 菜单弹层窗口复用同一份前端，只渲染菜单内容。
const label = getCurrentWindow().label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {label === "menu" ? (
      <MenuWindow />
    ) : label === "tab-menu" ? (
      <TabMenuWindow />
    ) : (
      <App />
    )}
  </React.StrictMode>,
);
