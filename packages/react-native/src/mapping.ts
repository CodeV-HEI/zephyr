export function declarationsToRN(
    declarations: Record<string, string>,
    baseFontSize = 16
): Record<string, unknown> {
    const result: Record<string, unknown> = {};
    for (const [prop, value] of Object.entries(declarations)) {
        const rnProp = prop.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
        // Skip 'float' – not supported in RN
        if (rnProp === 'float') continue;
        result[rnProp] = convertValue(value, baseFontSize);
    }
    return result;
}

function convertValue(value: string, baseFontSize: number): string | number {
    // Unitless 0
    if (value === '0') return 0;
    // Percentages
    if (/^-?\d+(\.\d+)?%$/.test(value)) return value;
    // rem → px
    const rem = value.match(/^(-?[\d.]+)rem$/);
    if (rem) return parseFloat(rem[1]) * baseFontSize;
    // px
    const px = value.match(/^(-?[\d.]+)px$/);
    if (px) return parseFloat(px[1]);
    // em – assume 1em = baseFontSize
    const em = value.match(/^(-?[\d.]+)em$/);
    if (em) return parseFloat(em[1]) * baseFontSize;
    // ch (approximate: 1ch ≈ 0.5em)
    const ch = value.match(/^(-?[\d.]+)ch$/);
    if (ch) return parseFloat(ch[1]) * baseFontSize * 0.5;
    // vw, vh – convert to percentage of screen? We'll leave as string; user can handle
    // Otherwise return raw string (including CSS variables like `var(--spacing-4)`)
    return value;
}