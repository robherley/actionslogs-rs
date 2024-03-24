import { useEffect, useState } from "react";

import Parser from "../parser";

export function useParser(): Parser | null {
    const [parser, setParser] = useState<Parser | null>(null);

    useEffect(() => {
        const initialize = async () => {
            await Parser.init();
            setParser(new Parser());
        };

        initialize();
    }, []);

    return parser;
}
