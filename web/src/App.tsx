import { useEffect, useState } from "react";
import { SearchIcon, HourglassIcon } from "@primer/octicons-react";
import {
    Box,
    Header,
    TextInput,
    Textarea,
    Text,
    themeGet,
    useTheme,
} from "@primer/react";
import ReactJson from "@microlink/react-json-view";

import init, { Parser } from "../gen/actionslogs";

const App = () => {
    const theme = useTheme();
    const [parser, setParser] = useState<Parser | null>(null);
    const [raw, setRaw] = useState<string>("{}");
    const [perf, setPerf] = useState<number>(0);
    const [matches, setMatches] = useState<number>(0);

    const stopWatch = (fn: () => void) => {
        const start = performance.now();
        fn();
        setPerf(performance.now() - start);
    };

    useEffect(() => {
        init().then(() => {
            setParser(new Parser());
        });
    }, []);

    if (!parser) return null;

    return (
        <>
            <Header>
                <Header.Item>
                    <Header.Link href="https://github.com/robherley/actionslogs-rs">
                        <span>Actions Log Parser</span>
                    </Header.Link>
                </Header.Item>
            </Header>
            <Box
                sx={{
                    margin: "0 auto",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    padding: "2rem",
                    maxWidth: "800px",
                }}
            >
                <Box
                    sx={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        width: "100%",
                        marginBottom: themeGet("space.2"),
                    }}
                >
                    <Box
                        color="fg.muted"
                        sx={{ display: "flex", alignItems: "center" }}
                    >
                        <HourglassIcon size={12} />
                        <Text sx={{ marginLeft: themeGet("space.1") }}>
                            {perf} ms
                        </Text>
                    </Box>
                    <TextInput
                        leadingVisual={SearchIcon}
                        aria-label="search"
                        name="search"
                        placeholder="Search"
                        autoComplete="off"
                        spellCheck="false"
                        trailingVisual={matches > 0 && matches}
                        onChange={({ target: { value } }) =>
                            stopWatch(() => {
                                parser.setSearch(value);
                                setRaw(parser.stringify(false));
                                setMatches(parser.getMatches());
                            })
                        }
                    />
                </Box>
                <Textarea
                    sx={{ marginBottom: themeGet("space.4") }}
                    rows={10}
                    cols={100}
                    spellCheck="false"
                    placeholder="##[info]hello world!"
                    onChange={({ target: { value } }) =>
                        stopWatch(() => {
                            parser.setRaw(value);
                            setRaw(parser.stringify(false));
                            setMatches(parser.getMatches());
                        })
                    }
                />
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
            </Box>
        </>
    );
};

export default App;
