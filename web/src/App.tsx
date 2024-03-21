import { useEffect, useState } from "react";

import init, { Parser } from "../gen/actionslogs";

function App() {
    const [parser, setParser] = useState<Parser | null>(null);
    const [raw, setRaw] = useState<string>("");
    const [perf, setPerf] = useState<number>(-1);
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
        <div className="main">
            <h1>Actions Log Parser</h1>
            <div className="search">
                <span>{matches > 0 && matches}</span>
                <label htmlFor="search">🔍</label>
                <input
                    id="search"
                    type="text"
                    placeholder="Search"
                    onChange={({ target: { value } }) =>
                        stopWatch(() => {
                            parser.setSearch(value);
                            setRaw(parser.stringify(true));
                            setMatches(parser.getMatches());
                        })
                    }
                />
            </div>
            <textarea
                rows={10}
                cols={100}
                spellCheck="false"
                placeholder="##[info]hello world!"
                onChange={({ target: { value } }) =>
                    stopWatch(() => {
                        parser.setRaw(value);
                        setRaw(parser.stringify(true));
                    })
                }
            ></textarea>
            <div>{perf >= 0 && `⏱️ ${perf} ms`}</div>
            <pre>{raw}</pre>
        </div>
    );
}

export default App;
