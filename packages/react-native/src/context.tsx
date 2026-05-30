import React, { createContext, useContext, useMemo } from 'react';
import { useColorScheme, useWindowDimensions } from 'react-native';
import type { ResolvedStyles } from './types';
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
    precompiledStyles: Record<string, ResolvedStyles>;
}

export function ZephyrProvider({
    baseFontSize = 16,
    children,
    precompiledStyles,
}: ZephyrProviderProps) {
    const colorScheme = useColorScheme();
    const { width, height } = useWindowDimensions();

    const cache = useMemo(() => new Map<string, Record<string, unknown>>(), []);

    // Determine which variant conditions are active
    const activeConditions = useMemo(() => {
        const conditions: Set<string> = new Set();
        // Color scheme
        if (colorScheme === 'dark') conditions.add('dark');
        else if (colorScheme === 'light') conditions.add('light');
        // Screen size (simplified – you can extend)
        if (width >= 768) conditions.add('md');
        if (width >= 1024) conditions.add('lg');
        if (width >= 1280) conditions.add('xl');
        if (width >= 1536) conditions.add('2xl');
        // You can also add orientation, high contrast, etc.
        return conditions;
    }, [colorScheme, width]);

    const resolveSync = (classString: string): Record<string, unknown> => {
        if (cache.has(classString)) return cache.get(classString)!;

        const resolved = precompiledStyles[classString];
        if (!resolved) {
            throw new Error(
                `Class "${classString}" not precompiled. Use Babel plugin to precompile.`
            );
        }

        // Start with base styles
        let style = declarationsToRN(resolved.base, baseFontSize);

        // Apply matching variants in order
        for (const variant of resolved.variants) {
            const isActive = variant.conditions.every(cond => {
                if (cond.type === 'pseudo') {
                    // Only "hover", "focus", etc. are possible; RN doesn't support them natively,
                    // but you can optionally ignore or map to Pressable states.
                    return false; // not supported yet
                }
                if (cond.type === 'selector') {
                    // e.g., "dark:bg-red" – extract the condition part (before colon)
                    const [condition] = cond.query.split(':');
                    return activeConditions.has(condition);
                }
                if (cond.type === 'media') {
                    // Evaluate media query (simplified)
                    if (cond.query.includes('prefers-color-scheme: dark')) {
                        return colorScheme === 'dark';
                    }
                    if (cond.query.includes('prefers-color-scheme: light')) {
                        return colorScheme === 'light';
                    }
                    if (cond.query.includes('(min-width:') || cond.query.includes('(max-width:')) {
                        // Very basic extraction – for real use use a library like 'css-mediaquery'
                        if (cond.query.includes('min-width: 768px')) return width >= 768;
                        if (cond.query.includes('min-width: 1024px')) return width >= 1024;
                        if (cond.query.includes('min-width: 1280px')) return width >= 1280;
                        if (cond.query.includes('min-width: 1536px')) return width >= 1536;
                    }
                    return false;
                }
                return false;
            });
            if (isActive) {
                style = { ...style, ...declarationsToRN(variant.style, baseFontSize) };
            }
        }

        cache.set(classString, style);
        return style;
    };

    return (
        <ZephyrContext.Provider value={{ resolveSync, baseFontSize }}>
            {children}
        </ZephyrContext.Provider>
    );
}