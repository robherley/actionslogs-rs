import { useEffect, useState } from "react";

import init, {Parser} from "../../gen/actionslogs";

export function useParser(): Parser | null {
    const [parser, setParser] = useState<Parser | null>(null);

    useEffect(() => {
        const initialize = async () => {
          await init()
          setParser(new Parser());
        }

        initialize();
    }, []);

    return parser;
}
