import { useState } from "react";
import {
    Box,
    Flash,
    SegmentedControl,
    useTheme,
    themeGet,
} from "@primer/react";
import ReactJson from "@microlink/react-json-view";

interface PreviewProps {
    raw: string;
}

const JSObject: React.FC<PreviewProps> = ({ raw }) => {
    const theme = useTheme();

    try {
        return (
            <ReactJson
                src={JSON.parse(raw)}
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

const Rendered: React.FC<PreviewProps> = () => {
    return <Box>Coming soon!</Box>;
};

const Preview: React.FC<{ raw: string }> = ({ raw }) => {
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
            <PreviewComponent raw={raw} />
        </Box>
    );
};

export default Preview;
