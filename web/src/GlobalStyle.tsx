import { themeGet } from "@primer/react";
import { createGlobalStyle } from "styled-components";

const GlobalStyle = createGlobalStyle`
  body {
    background-color: ${themeGet("colors.canvas.default")};
  }
`;

export default GlobalStyle;
