import type { Condition, ResolvedStyles } from './types';

export function parseCompiledCSS(css: string): ResolvedStyles {
    const base: Record<string, string> = {};
    const variants: Array<{ conditions: Condition[]; style: Record<string, string> }> = [];

    // Extraction simplifiée : on ignore les @media pour l'instant, on récupère toutes les règles
    const ruleRegex = /([^{}]+)\{([^{}]+)\}/g;
    let match;
    while ((match = ruleRegex.exec(css)) !== null) {
        const selector = match[1].trim();
        const decls = match[2].trim();
        if (selector.startsWith('@media')) {
            // Extraire la condition
            const mediaMatch = selector.match(/@media\s+(.+)/);
            if (mediaMatch) {
                const condition: Condition = { type: 'media', query: mediaMatch[1] };
                const style: Record<string, string> = {};
                decls.split(';').forEach(decl => {
                    const [prop, val] = decl.split(':');
                    if (prop && val) style[prop.trim()] = val.trim();
                });
                variants.push({ conditions: [condition], style });
            }
        } else {
            // Règle normale (classe unique)
            decls.split(';').forEach(decl => {
                const [prop, val] = decl.split(':');
                if (prop && val) base[prop.trim()] = val.trim();
            });
        }
    }
    return { base, variants };
}