import { useState } from "react";
import { SearchIcon, HourglassIcon } from "@primer/octicons-react";
import { Box, TextInput, Textarea, Text, themeGet } from "@primer/react";

import Layout from "./components/Layout";
import Preview from "./components/Preview";
import { useParser } from "./hooks/useParser";

const App = () => {
    const parser = useParser();
    const [raw, setRaw] = useState<string>("{}");
    const [perf, setPerf] = useState<number>(0);
    const [matches, setMatches] = useState<number>(0);

    const stopWatch = (fn: () => void) => {
        const start = performance.now();
        fn();
        setPerf(performance.now() - start);
    };

    if (!parser) return null;

    return (
        <Layout>
            <Box
                sx={{
                    margin: "0 auto",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    padding: "2rem",
                    maxWidth: "1000px",
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
                    style={{ resize: "none" }}
                    rows={10}
                    cols={1000}
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

                <Preview raw={raw} />
            </Box>
        </Layout>
    );
};

export default App;
