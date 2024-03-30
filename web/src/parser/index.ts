import * as actionslogs from "../../gen/actionslogs/actionslogs";
import type { InitInput } from "../../gen/actionslogs/actionslogs";

export interface Line {
    n: number;
    ts: number;
    cmd?: Command;
    elements: Element[];
    group?: Group;
}

export interface Group {
    children: Line[];
    ended: boolean;
}

export enum Command {
    Command = 1,
    Debug = 2,
    Error = 3,
    Info = 4,
    Notice = 5,
    Verbose = 6,
    Warning = 7,
    Group = 8,
    EndGroup = 9,
}

export type Element = TextElement | LinkElement | string;

export interface TextElement {
    content: string;
    styles: Styles;
}

export interface LinkElement {
    href: string;
    children: Element[];
}

export interface Styles {
    b?: boolean;
    i?: boolean;
    u?: boolean;
    hl?: boolean;
    fg?: Color;
    bg?: Color;
}

export type Color = number | [number, number, number];

class Parser extends actionslogs.Parser {
    constructor() {
        super();
    }

    lines(): Line[] {
        const parsed = JSON.parse(this.stringify(false));
        if (!Array.isArray(parsed)) {
            throw new TypeError("expected array");
        }

        // TODO: maybe validate a little bit
        return parsed as Line[];
    }

    static async init(
        modOrPath?: InitInput | Promise<InitInput>
    ): Promise<InitInput> {
        return actionslogs.default(modOrPath);
    }
}

export default Parser;
