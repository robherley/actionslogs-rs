import { Header, Text, themeGet } from "@primer/react";
import { CodeIcon } from "@primer/octicons-react";
import { createGlobalStyle } from "styled-components";

export const Style = createGlobalStyle`
  body {
    background-color: ${themeGet("colors.canvas.default")};
  }
`;

const Layout: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <>
        <Header>
            <Header.Item>
                <Text as="strong">Actions Log Parser</Text>
                <Header.Link
                    href="https://github.com/robherley/actionslogs-rs"
                    sx={{ color: "fg.muted", marginLeft: themeGet("space.2") }}
                >
                    <CodeIcon size={16} />
                </Header.Link>
            </Header.Item>
        </Header>
        {children}
    </>
);

export default Layout;
