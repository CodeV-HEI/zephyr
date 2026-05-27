import React, { createContext, useContext, useEffect, useState } from 'react';
import { compileClasses, initZephyrCore, ResolvedStyles } from '@zephyr/core';
import { declarationsToRN } from './mapping';

interface ZephyrContextValue {
    resolveSync: (classString: string) => Record<string, unknown>;
    baseFontSize: number;
}

const ZephyrContext = createContext<ZephyrContextValue | null>(null);

export function useZephyrContext() {
    const ctx = useContext(ZephyrContext);
    if (!ctx) throw new Error('ZephyrProvider is missing');
    return ctx;
}

interface ZephyrProviderProps {
    baseFontSize?: number;
    children: React.ReactNode;
    precompiledStyles?: Map<string, ResolvedStyles>; // pré‑compilation statique
}

export function ZephyrProvider({
    baseFontSize = 16,
    children,
    precompiledStyles,
}: ZephyrProviderProps) {
    const [cache, setCache] = useState<Map<string, Record<string, unknown>>>(new Map());
    const [ready, setReady] = useState(false);

    useEffect(() => {
        initZephyrCore().then(() => setReady(true));
    }, []);

    const resolveSync = (classString: string): Record<string, unknown> => {
        if (!ready) throw new Error('Zephyr core not initialized yet');
        if (cache.has(classString)) return cache.get(classString)!;

        // Si on a des styles pré‑compilés (build‑time)
        if (precompiledStyles?.has(classString)) {
            const resolved = precompiledStyles.get(classString)!;
            const style = declarationsToRN(resolved.base, baseFontSize);
            cache.set(classString, style);
            return style;
        }

        // Sinon, compilation à la volée (déconseillé en production)
        // On lance la compilation asynchrone et on la met en cache pour les prochains rendus
        // Pour une résolution synchrone immédiate, on lève une exception.
        // Idéalement, on doit pré‑compiler toutes les classes utilisées (via un plugin Babel/Metro).
        throw new Error(
            `Class "${classString}" not precompiled. Use Babel plugin or provide precompiledStyles.`
        );
    };

    if (!ready) return null; // ou un loader

    return (
        <ZephyrContext.Provider value={{ resolveSync, baseFontSize }}>
            {children}
        </ZephyrContext.Provider>
    );
}