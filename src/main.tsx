import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { QueryClientContainer } from "./providers/query-client-provider";
import { ThemeProvider } from "./providers/theme-provider";
import { ToastProvider } from "./providers/toast-provider";
import "./styles/globals.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <QueryClientContainer>
        <ToastProvider>
          <App />
        </ToastProvider>
      </QueryClientContainer>
    </ThemeProvider>
  </React.StrictMode>,
);
