import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider, BaseStyles } from "@primer/react";

import App from "./App.tsx";
import GlobalStyle from "./GlobalStyle.tsx";

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <ThemeProvider colorMode="auto">
            <GlobalStyle />
            <BaseStyles>
                <App />
            </BaseStyles>
        </ThemeProvider>
    </React.StrictMode>
);
