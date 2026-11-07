import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
// Import Radix Themes piecewise: the full styles.css ships every color scale
// (~180 KB of tokens) while the app only uses the scales listed here.
import "@radix-ui/themes/tokens/base.css";
import "@radix-ui/themes/tokens/colors/gray.css";
import "@radix-ui/themes/tokens/colors/grass.css";
import "@radix-ui/themes/tokens/colors/blue.css";
import "@radix-ui/themes/tokens/colors/green.css";
import "@radix-ui/themes/tokens/colors/orange.css";
import "@radix-ui/themes/tokens/colors/red.css";
import "@radix-ui/themes/components.css";
import "@radix-ui/themes/utilities.css";
import "./design-system/tokens.css";
import "./design-system/primitives.css";
import "./design-system/web-adapter.css";
import "./globals.css";
import "./workspace.css";

async function enableMocking() {
  if (!import.meta.env.DEV) {
    return;
  }

  const { worker } = await import("./mocks/browser");
  return worker.start();
}

enableMocking().then(() => {
  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
});
