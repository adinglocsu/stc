// @module: amd
// @traceResolution: true

// @filename: c:/root/src/tsconfig.json
{
    "compilerOptions": {
        "rootDirs": [
            ".",
            "../generated/src"
        ]
    }
}

// @filename: c:/root/src/file1.ts
import {x} from "./project/file3";
declare function use(x: string);
use(x.toExponential());

// @filename: c:/root/src/file2.d.ts
export let x: number;

// @filename: c:/root/generated/src/project/file3.ts
export {x} from "../file2";