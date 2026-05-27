import * as wasmModule from '../pkg/zephyr_core.js';
import { parseCompiledCSS } from './parser.js';
import type { ResolvedStyles } from './types.js';

// Récupère la fonction d'initialisation (default ou nommée)
const init = (wasmModule as any).default || (wasmModule as any).init;
const compile_tailwind_classes = (wasmModule as any).compile_tailwind_classes;

if (!init || !compile_tailwind_classes) {
    throw new Error('Invalid WASM module: missing init or compile_tailwind_classes');
}

let wasmReady: Promise<void> | null = null;

export async function initZephyrCore() {
    if (!wasmReady) {
        wasmReady = init();
        await wasmReady;
    }
}

export async function compileClasses(classString: string): Promise<ResolvedStyles> {
    await initZephyrCore();
    const json = compile_tailwind_classes(classString);
    const { css } = JSON.parse(json as string) as { css: string };
    return parseCompiledCSS(css);
}

export { parseCompiledCSS };
export type { ResolvedStyles, Condition } from './types.js';