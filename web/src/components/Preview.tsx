import { useState } from "react";
import {
    Box,
    Flash,
    SegmentedControl,
    useTheme,
    themeGet,
    Token,
} from "@primer/react";
import ReactJson from "@microlink/react-json-view";
import type { Line, Element, Color } from "../parser";

interface PreviewProps {
    lines: Line[];
}

const JSObject: React.FC<PreviewProps> = ({ lines }) => {
    const theme = useTheme();

    try {
        return (
            <ReactJson
                name={false}
                src={lines}
                iconStyle="circle"
                theme={
                    ["night", "dark"].includes(theme.resolvedColorMode!)
                        ? "bright"
                        : "bright:inverted"
                }
                style={{ width: "100%", backgroundColor: "transparent" }}
                collapsed={false}
                enableClipboard={false}
                displayDataTypes={false}
                displayObjectSize={false}
            />
        );
    } catch (e: unknown) {
        return (
            <Flash variant="danger">
                {e instanceof Error
                    ? e.message
                    : "An error occurred while parsing the JSON object."}
            </Flash>
        );
    }
};

const colorToCSS = (color: Color): string => {
    if (Array.isArray(color)) {
        return `rgb(${color.join(",")})`;
    }

    return (
        [
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
            "gray",
            "dark gray",
            "light red",
            "light green",
            "light yellow",
            "light blue",
            "light magenta",
            "light cyan",
            "light white",
        ][color] || "inherit"
    );
};

const renderElement = (element: Element): JSX.Element | null => {
    if (typeof element === "string") {
        return <span>{element}</span>;
    }

    if ("href" in element) {
        return (
            <a href={element.href}>
                {element.children.map((child) => renderElement(child))}
            </a>
        );
    }

    if ("content" in element) {
        const style: React.CSSProperties = {};

        if (element.styles.b) {
            style.fontWeight = "bold";
        }

        if (element.styles.i) {
            style.fontStyle = "italic";
        }

        if (element.styles.u) {
            style.textDecoration = "underline";
        }

        if (element.styles.hl) {
            style.color = "yellow";
        }

        if (element.styles.fg) {
            style.color = colorToCSS(element.styles.fg);
        }

        if (element.styles.bg) {
            style.backgroundColor = colorToCSS(element.styles.bg);
        }
        return <span style={style}>{element.content}</span>;
    }

    return null;
};

const renderLine = (line: Line): JSX.Element => {
    if (line.group) {
        return (
            <>
                <details key={line.n}>
                    <summary>
                        {line.n} {line.elements.map(renderElement)}
                    </summary>
                    {line.group.children.map(renderLine)}
                </details>
            </>
        );
    }

    return (
        <div key={line.n}>
            {line.n} {line.elements.map(renderElement)}
        </div>
    );
};

const Rendered: React.FC<PreviewProps> = ({ lines }) => {
    return (
        <>
            <div>
                <Token text="Work in Progress" />
            </div>
            <pre style={{ whiteSpace: "pre-wrap", wordBreak: "break-word" }}>
                {lines.map(renderLine)}
            </pre>
        </>
    );
};

const Preview: React.FC<PreviewProps> = ({ lines }) => {
    const [switcher, setSwitcher] = useState<number>(1);

    const controls: { label: string; component: React.FC<PreviewProps> }[] = [
        {
            label: "Rendered",
            component: Rendered,
        },
        {
            label: "JSON",
            component: JSObject,
        },
    ];

    const PreviewComponent = controls[switcher]?.component;

    return (
        <Box sx={{ alignSelf: "flex-start" }}>
            <SegmentedControl
                aria-label="Log view"
                onChange={setSwitcher}
                sx={{
                    marginBottom: themeGet("space.4"),
                }}
            >
                {controls.map((control, idx) => (
                    <SegmentedControl.Button
                        key={control.label}
                        selected={switcher == idx}
                    >
                        {control.label}
                    </SegmentedControl.Button>
                ))}
            </SegmentedControl>
            <PreviewComponent lines={lines} />
        </Box>
    );
};

export default Preview;
